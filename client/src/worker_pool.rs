use std::{
    collections::HashSet,
    hash::Hash,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};

use futures::{SinkExt, Stream, StreamExt};
use image::Rgb32FImage;
use tokio::{
    net::{TcpStream, UdpSocket},
    sync::watch,
};
use tokio_stream::wrappers::WatchStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use worker::{
    RenderedImage,
    api::render_task::RenderTask,
    discovery::{Request as DiscoveryRequest, Response as DiscoveryResponse},
};

use crate::frame::Frame;

const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(5);

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct WorkerPool {
    workers: HashSet<Worker>,
    stats_sender: StatsSender,
}

struct StatsSender {
    workers: watch::Sender<Vec<SocketAddr>>,
}

impl StatsSender {
    fn notify_new_worker_addresses(&self, addresses: Vec<SocketAddr>) {
        self.workers.send(addresses).unwrap();
    }
}

impl Default for StatsSender {
    fn default() -> Self {
        Self {
            workers: watch::Sender::new(vec![]),
        }
    }
}

#[derive(Clone)]
pub struct Stats {
    workers: watch::Receiver<Vec<SocketAddr>>,
}

impl Hash for Stats {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}

impl Stats {
    fn from_sender(sender: &StatsSender) -> Self {
        Self {
            workers: sender.workers.subscribe(),
        }
    }

    pub fn get_worker_addresses_stream(self) -> impl Stream<Item = Vec<SocketAddr>> {
        WatchStream::new(self.workers.clone())
    }
}

impl WorkerPool {
    pub fn new() -> WorkerPool {
        WorkerPool {
            workers: HashSet::new(),
            stats_sender: StatsSender::default(),
        }
    }

    pub fn get_stats(&self) -> Stats {
        Stats::from_sender(&self.stats_sender)
    }

    pub async fn discover(&mut self, port: u16) {
        let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::from_octets([0, 0, 0, 0]), 0))
            .await
            .unwrap();
        socket.set_broadcast(true).unwrap();

        let request = DiscoveryRequest {};
        let request = postcard::to_stdvec(&request).unwrap();

        socket
            .send_to(&request, SocketAddrV4::new(Ipv4Addr::BROADCAST, port))
            .await
            .unwrap();

        tokio::select! {
            _ = tokio::time::sleep(DISCOVERY_TIMEOUT) => {},
            _ = self.listen_for_workers(&socket) => {}
        };
    }

    async fn listen_for_workers(&mut self, socket: &UdpSocket) {
        // TODO: Get the size of response.
        let mut buf = vec![0; 1024];
        let (length, mut worker_address) = socket.recv_from(&mut buf[..]).await.unwrap();

        let response: DiscoveryResponse = postcard::from_bytes(&buf[..length]).unwrap();
        worker_address.set_port(response.websocket_port);

        let worker = Worker {
            address: worker_address,
        };
        self.workers.insert(worker);

        self.stats_sender.notify_new_worker_addresses(
            self.workers.iter().map(|worker| worker.address).collect(),
        );

        println!("Worker discovered: {}", worker_address);
    }

    pub async fn send_render_task(&self, render_task: RenderTask, frame: Arc<Frame>) {
        // TODO: Distribute render tasks.
        for worker in &self.workers {
            worker.get_image(render_task.clone(), frame.clone()).await;
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
struct Worker {
    address: SocketAddr,
}

impl Worker {
    async fn get_image(&self, render_task: RenderTask, frame: Arc<Frame>) {
        // TODO: Don't drop connection.
        let mut connection = connect(self.address).await;
        let image = get_image(&mut connection, render_task).await;
        frame.add_render(image).await;
    }
}

async fn connect(address: SocketAddr) -> WsStream {
    let url = format!("ws://{}", address);
    println!("Connecting to worker {}", url);
    connect_async(url).await.unwrap().0
}

async fn get_image(connection: &mut WsStream, render_task: RenderTask) -> Rgb32FImage {
    let render_task = serde_json::to_string(&render_task).unwrap();
    connection.send(Message::text(render_task)).await.unwrap();

    let image = connection.next().await.unwrap().unwrap();
    let Message::Binary(image) = image else {
        panic!("Wrong message format");
    };

    RenderedImage::from_bytes(image.to_vec()).image
}
