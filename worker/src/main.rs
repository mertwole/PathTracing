use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

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

const RENDER_TASKS_CHANNEL_BUFFER: usize = 8;
const IMAGES_CHANNEL_BUFFER: usize = 8;
const FILE_REQUESTS_CHANNEL_BUFFER: usize = 8;

type WsStream = WebSocketStream<TcpStream>;

#[derive(Parser)]
pub struct Cli {}

#[tokio::main]
async fn main() {
    let _args = Cli::parse();

    let addr = format!("0.0.0.0:{}", WEBSOCKET_PORT);

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    tokio::spawn(listen_discovery_broadcasts());

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
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

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let channel = WebSocketChannel::new(ws_stream);
    channel.run().await;
}

struct WebSocketChannel {
    sender: MessageSender,
    receiver: MessageReceiver,
    work_processor: WorkProcessor,
}

impl WebSocketChannel {
    fn new(connection: WsStream) -> Self {
        let (sink, stream) = connection.split();

        let (render_tasks_sender, render_tasks_receiver) =
            mpsc::channel(RENDER_TASKS_CHANNEL_BUFFER);
        let (images_sender, images_receiver) = mpsc::channel(IMAGES_CHANNEL_BUFFER);
        let (file_requests_sender, file_requests_receiver) =
            mpsc::channel(FILE_REQUESTS_CHANNEL_BUFFER);

        let files_fetcher = WsFileFetcher::new(file_requests_sender);

        Self {
            sender: MessageSender {
                sink,
                file_requests: file_requests_receiver,
                images: images_receiver,
            },
            receiver: MessageReceiver {
                stream,
                files_fetcher: files_fetcher.clone(),
                render_tasks: render_tasks_sender,
            },
            work_processor: WorkProcessor {
                files_fetcher,
                render_tasks: render_tasks_receiver,
                images: images_sender,
            },
        }
    }

    async fn run(self) {
        let Self {
            sender,
            receiver,
            work_processor,
        } = self;

        tokio::spawn(work_processor.run());
        tokio::spawn(sender.start_sending());
        tokio::spawn(receiver.start_receiving());
    }
}

struct MessageSender {
    sink: SplitSink<WsStream, Message>,

    file_requests: mpsc::Receiver<String>,
    images: mpsc::Receiver<RenderedImage>,
}

impl MessageSender {
    async fn start_sending(mut self) {
        loop {
            let message = tokio::select! {
                file_request = self.file_requests.recv() => {
                    WebSocketMessageOut::FileRequest { path: file_request.unwrap() }
                },
                image = self.images.recv() => {
                    WebSocketMessageOut::Render(Box::new(image.unwrap()))
                }
            };

            self.sink.send(message.serialize()).await.unwrap();
        }
    }
}

struct MessageReceiver {
    stream: SplitStream<WsStream>,
    files_fetcher: WsFileFetcher,

    render_tasks: mpsc::Sender<RenderTask>,
}

impl MessageReceiver {
    async fn start_receiving(mut self) {
        loop {
            let message = self.stream.next().await.unwrap().unwrap();
            let message = WebSocketMessageIn::deserialize(message);

            match message {
                WebSocketMessageIn::File { content, path } => {
                    self.files_fetcher.provide_file(path, content).await;
                }
                WebSocketMessageIn::RenderTask(task) => {
                    self.render_tasks.send(*task).await.unwrap();
                }
            }
        }
    }
}

struct WorkProcessor {
    files_fetcher: WsFileFetcher,

    render_tasks: mpsc::Receiver<RenderTask>,
    images: mpsc::Sender<RenderedImage>,
}

impl WorkProcessor {
    async fn run(mut self) {
        let mut worker = Worker::new();

        loop {
            let task = self.render_tasks.recv().await.unwrap();
            let image = worker.render(task, &self.files_fetcher).await;
            self.images.send(RenderedImage { image }).await.unwrap();
        }
    }
}

#[derive(Clone)]
struct WsFileFetcher {
    received_files: Arc<Mutex<HashMap<String, Vec<u8>>>>,

    file_requests: mpsc::Sender<String>,
}

impl FileFetcher for WsFileFetcher {
    async fn fetch(&self, path: &str) -> Vec<u8> {
        self.file_requests.send(path.to_string()).await.unwrap();

        loop {
            let mut files = self.received_files.lock().await;
            if let Some(content) = files.remove(path) {
                return content;
            };

            // TODO: Find a smarter way of doing this.
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

impl WsFileFetcher {
    fn new(file_requests: mpsc::Sender<String>) -> Self {
        Self {
            received_files: Default::default(),
            file_requests,
        }
    }

    async fn provide_file(&self, path: String, content: Vec<u8>) {
        let mut files = self.received_files.lock().await;
        files.insert(path, content);
    }
}
