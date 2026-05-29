use std::{
    collections::HashSet,
    hash::Hash,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};

use futures::{SinkExt, StreamExt};
use image::Rgb32FImage;
use tokio::{
    net::{TcpStream, UdpSocket},
    sync::{RwLock, mpsc, watch},
    time::timeout,
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
const DISCOVERY_PORT: u16 = 40000;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub fn start(frame: Arc<Frame>) -> Handle {
    let (discovery_requests_sender, discovery_requests_receiver) = watch::channel(());
    let (discovered_workers_sender, discovered_workers_receiver) = watch::channel(vec![]);
    let (render_tasks_sender, render_tasks_receiver) = mpsc::channel(1);

    let pool = Pool::new(
        frame,
        discovery_requests_receiver,
        discovered_workers_sender,
        render_tasks_receiver,
    );
    tokio::spawn(pool.run());

    Handle {
        discovery_requests: discovery_requests_sender,
        render_tasks_queue: render_tasks_sender,
        discovered_workers: discovered_workers_receiver,
    }
}

#[derive(Clone)]
pub struct Handle {
    discovery_requests: watch::Sender<()>,
    render_tasks_queue: mpsc::Sender<RenderTask>,
    discovered_workers: watch::Receiver<Vec<SocketAddr>>,
}

impl Hash for Handle {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}

impl Handle {
    pub fn discover(&self) {
        self.discovery_requests.send(()).unwrap();
    }

    pub fn send_render_task(&self, render_task: RenderTask) -> Result<(), ()> {
        // TODO: Properly handle 2 types of error here.
        self.render_tasks_queue
            .try_send(render_task)
            .map_err(|_| ())
    }

    pub fn get_worker_discovery_stream(&self) -> WatchStream<Vec<SocketAddr>> {
        WatchStream::new(self.discovered_workers.clone())
    }
}

struct Pool {
    finder: Finder,
    scheduler: Scheduler,
}

impl Pool {
    fn new(
        frame: Arc<Frame>,
        discovery_requests: watch::Receiver<()>,
        dicovered_workers_sender: watch::Sender<Vec<SocketAddr>>,
        render_tasks: mpsc::Receiver<RenderTask>,
    ) -> Pool {
        let workers = Arc::from(RwLock::new(HashSet::new()));

        let finder = Finder {
            workers: workers.clone(),
            discovery_requests,
            dicovered_workers_sender,
        };
        let scheduler = Scheduler {
            workers: workers.clone(),
            frame,
            render_tasks,
        };

        Pool { finder, scheduler }
    }

    async fn run(self) {
        let Self {
            finder,
            mut scheduler,
        } = self;

        tokio::select! {
            _ = finder.run_discovery(DISCOVERY_PORT) => {},
            _ = scheduler.schedule_render_tasks() => {}
        };
    }
}

struct Finder {
    workers: Arc<RwLock<HashSet<Worker>>>,

    discovery_requests: watch::Receiver<()>,
    dicovered_workers_sender: watch::Sender<Vec<SocketAddr>>,
}

impl Finder {
    async fn run_discovery(&self, port: u16) {
        let mut discovery_requests = self.discovery_requests.clone();
        loop {
            discovery_requests.changed().await.unwrap();

            println!("Discovering workers...");

            self.discover(port).await;
        }
    }

    async fn discover(&self, port: u16) {
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

        let _ = timeout(DISCOVERY_TIMEOUT, async {
            loop {
                self.listen_for_workers(&socket).await
            }
        })
        .await;
    }

    async fn listen_for_workers(&self, socket: &UdpSocket) {
        // TODO: Get the size of response.
        let mut buf = vec![0; 1024];
        let (length, mut worker_address) = socket.recv_from(&mut buf[..]).await.unwrap();

        let response: DiscoveryResponse = postcard::from_bytes(&buf[..length]).unwrap();
        worker_address.set_port(response.websocket_port);

        let worker = Worker {
            address: worker_address,
        };

        println!("Worker discovered: {}", worker_address);

        let mut workers = self.workers.read().await.clone();
        workers.insert(worker);
        *(self.workers.write().await) = workers.clone();

        self.dicovered_workers_sender
            .send(workers.iter().map(|worker| worker.address).collect())
            .unwrap();
    }
}

struct Scheduler {
    workers: Arc<RwLock<HashSet<Worker>>>,
    frame: Arc<Frame>,

    render_tasks: mpsc::Receiver<RenderTask>,
}

impl Scheduler {
    async fn schedule_render_tasks(&mut self) {
        loop {
            let task = self.render_tasks.recv().await.unwrap();

            let workers = self.workers.read().await.clone();
            // TODO: Distribute render tasks.
            for worker in workers {
                worker.get_image(task.clone(), self.frame.clone()).await;
            }
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
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
