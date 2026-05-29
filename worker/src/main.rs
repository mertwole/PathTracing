use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use clap::Parser;
use futures::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use tokio::{
    net::{TcpListener, TcpStream, UdpSocket},
    sync::Mutex,
};
use tokio_tungstenite::{WebSocketStream, tungstenite::protocol::Message};

use worker::{RenderedImage, Worker, api::render_task::RenderTask};

const WEBSOCKET_PORT: u16 = 30000;
const BROADCAST_PORT: u16 = 40000;

type WsStream = WebSocketStream<TcpStream>;

#[derive(Parser)]
pub struct Cli {
    #[clap(long)]
    mongodb_url: String,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let worker = Arc::from(Mutex::new(Worker::new(args.mongodb_url)));
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

    let Message::Text(message) = message else {
        anyhow::bail!("Unexpected message format");
    };
    let render_task: RenderTask = serde_json::from_str(&message)
        .map_err(|err| anyhow::anyhow!("Failed to decode render task: {}", err))?;

    let image = worker.lock().await.render(render_task).await;

    let image_data = RenderedImage { image }.to_bytes();
    let message = Message::binary(image_data);

    outgoing
        .send(message)
        .await
        .context("Failed to send render result")?;

    Ok(())
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
