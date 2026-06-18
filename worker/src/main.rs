use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use clap::Parser;
use futures::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use tokio::{
    net::{TcpListener, TcpStream, UdpSocket},
    sync::{Mutex, mpsc},
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

    loop {
        if let Err(err) = connection_loop(&mut outgoing, &mut incoming, worker.clone()).await {
            println!("Error during message exchange: {}", err);
            break;
        }
    }
}

async fn connection_loop(
    outgoing: &mut SplitSink<WsStream, Message>,
    incoming: &mut SplitStream<WsStream>,
    worker: Arc<Mutex<Worker>>,
) -> anyhow::Result<()> {
    // TODO: Process case when connection was gracefully closed.
    let message = incoming.next().await.unwrap()?;

    let message = WebSocketMessageIn::deserialize(message);

    match message {
        WebSocketMessageIn::File { content, path } => {
            //
        }
        WebSocketMessageIn::RenderTask(task) => {
            let image = worker.lock().await.render(task).await;

            let response = WebSocketMessageOut::Render(Box::from(RenderedImage { image }));

            outgoing
                .send(response.serialize())
                .await
                .context("Failed to send render result")?;
        }
    }

    Ok(())
}

pub struct WebSocketFileFetcher {}

impl WebSocketFileFetcher {
    fn new(file_requests: mpsc::Receiver<String>, files: mpsc::Sender<Vec<u8>>) -> Self {
        Self {}
    }

    fn on_file_received(&self, path: String, file: Vec<u8>) {
        //
    }
}

impl FileFetcher for WebSocketFileFetcher {
    async fn fetch(&self, path: &str) -> Vec<u8> {
        todo!()
    }
}

async fn process_render_tasks(
    tasks: mpsc::Receiver<RenderTask>,
    renders: mpsc::Sender<RenderedImage>,
) {
    loop {
        //
    }
}
