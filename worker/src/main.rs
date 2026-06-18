use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use clap::Parser;
use futures::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use tokio::{
    net::{TcpListener, TcpStream, UdpSocket},
    sync::{Mutex, broadcast, mpsc},
};
use tokio_tungstenite::{WebSocketStream, tungstenite::protocol::Message};

use worker::{
    RenderedImage, WebSocketMessageIn, WebSocketMessageOut, Worker, api::render_task::RenderTask,
    file_fetcher::FileFetcher,
};

const WEBSOCKET_PORT: u16 = 30000;
const BROADCAST_PORT: u16 = 40000;

type WsStream = WebSocketStream<TcpStream>;

#[derive(Parser)]
pub struct Cli {}

#[tokio::main]
async fn main() {
    let _args = Cli::parse();

    let worker = Arc::from(Mutex::new(Worker::new()));
    start_ws(worker).await;
}

async fn start_ws(worker: Arc<Mutex<Worker>>) {
    let addr = format!("0.0.0.0:{}", WEBSOCKET_PORT);

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    tokio::spawn(listen_discovery_broadcasts());

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr, worker.clone()));
    }
}

async fn listen_discovery_broadcasts() {
    let socket = UdpSocket::bind(&format!("0.0.0.0:{}", BROADCAST_PORT))
        .await
        .unwrap();
    socket.set_broadcast(true).unwrap();

    loop {
        // TODO: Determine len.
        let mut buffer = vec![0; 1024];
        let (len, sender) = socket.recv_from(&mut buffer[..]).await.unwrap();
        let _request: worker::discovery::Request = postcard::from_bytes(&buffer[..len]).unwrap();

        let response = worker::discovery::Response {
            websocket_port: WEBSOCKET_PORT,
        };
        let response = postcard::to_allocvec(&response).unwrap();
        socket.send_to(&response, sender).await.unwrap();
    }
}

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr, worker: Arc<Mutex<Worker>>) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let (mut outgoing, mut incoming) = ws_stream.split();

    let (render_tasks_sender, render_tasks_receiver) = mpsc::channel(10);
    let (renders_sender, _) = broadcast::channel(10);

    let file_fetcher = Arc::new(WebSocketFileFetcher::new());
    let file_fetcher_clone = file_fetcher.clone();

    let renders_sender_clone = renders_sender.clone();

    tokio::spawn(async move {
        process_render_tasks(
            render_tasks_receiver,
            renders_sender_clone,
            file_fetcher_clone,
        )
    });

    loop {
        if let Err(err) = connection_loop(
            &mut outgoing,
            &mut incoming,
            render_tasks_sender.clone(),
            renders_sender.clone().subscribe(),
            file_fetcher.clone(),
        )
        .await
        {
            println!("Error during message exchange: {}", err);
            break;
        }
    }
}

async fn connection_loop(
    outgoing: &mut SplitSink<WsStream, Message>,
    incoming: &mut SplitStream<WsStream>,
    tasks: mpsc::Sender<RenderTask>,
    mut renders: broadcast::Receiver<RenderedImage>,
    file_fetcher: Arc<WebSocketFileFetcher>,
) -> anyhow::Result<()> {
    tokio::select! {
        message = incoming.next() => {
            // TODO: Process case when connection was gracefully closed.
            let message = WebSocketMessageIn::deserialize(message.unwrap().unwrap());
            match message {
                WebSocketMessageIn::File { content, path } => {
                    file_fetcher.on_file_received(path, content).await;
                }
                WebSocketMessageIn::RenderTask(task) => {
                    tasks.send(*task).await.unwrap();
                }
            }
        },
        render = renders.recv() => {
            let message = WebSocketMessageOut::Render(Box::from(render.unwrap()));
            outgoing.send(message.serialize()).await.unwrap();
        }
    };

    Ok(())
}

pub struct WebSocketFileFetcher {
    received_files: Mutex<HashMap<String, Vec<u8>>>,
}

impl WebSocketFileFetcher {
    fn new() -> Self {
        Self {
            received_files: Default::default(),
        }
    }

    async fn on_file_received(&self, path: String, file: Vec<u8>) {
        self.received_files.lock().await.insert(path, file);
    }
}

impl FileFetcher for WebSocketFileFetcher {
    // TODO: Find a smarter way of implementing this.
    async fn fetch(&self, path: &str) -> Vec<u8> {
        // TODO: Send request for file to WS.
        loop {
            let mut received_files = self.received_files.lock().await;
            if !received_files.contains_key(path) {
                tokio::time::sleep(Duration::from_millis(10)).await;
                continue;
            }

            let file = received_files.remove(path).unwrap();
            return file;
        }
    }
}

async fn process_render_tasks(
    mut tasks: mpsc::Receiver<RenderTask>,
    renders: broadcast::Sender<RenderedImage>,
    file_fetcher: Arc<WebSocketFileFetcher>,
) {
    let mut worker = Worker::new();

    loop {
        let task = tasks.recv().await.unwrap();
        let image = worker.render(task, file_fetcher.as_ref()).await;
        renders.send(RenderedImage { image }).ok().unwrap();
    }
}
