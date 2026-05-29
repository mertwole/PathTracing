use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use futures::{SinkExt, StreamExt};
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
const WORKERS_DISCOVERY_CHANNEL_BUFFER: usize = 8;

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
        discovered_workers_watch: watch::Sender<Vec<SocketAddr>>,
        render_tasks: mpsc::Receiver<RenderTask>,
    ) -> Pool {
        let workers = Arc::from(RwLock::new(HashSet::new()));
        let (discovered_workers_sender, discovered_workers_receiver) =
            mpsc::channel(WORKERS_DISCOVERY_CHANNEL_BUFFER);

        let finder = Finder {
            workers: workers.clone(),
            discovery_requests,
            discovered_workers_watch,
            discovered_workers_sender,
        };
        let scheduler = Scheduler::new(discovered_workers_receiver, frame, render_tasks);

        Pool { finder, scheduler }
    }

    async fn run(self) {
        let Self { finder, scheduler } = self;

        tokio::select! {
            _ = finder.run_discovery(DISCOVERY_PORT) => {},
            _ = scheduler.schedule_render_tasks() => {}
        };
    }
}

struct Finder {
    workers: Arc<RwLock<HashSet<WorkerDescriptor>>>,

    discovery_requests: watch::Receiver<()>,
    discovered_workers_watch: watch::Sender<Vec<SocketAddr>>,
    discovered_workers_sender: mpsc::Sender<WorkerDescriptor>,
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

        let worker = WorkerDescriptor {
            address: worker_address,
        };

        println!("Worker discovered: {}", worker_address);

        let mut workers = self.workers.read().await.clone();
        workers.insert(worker.clone());
        *(self.workers.write().await) = workers.clone();

        self.discovered_workers_sender.send(worker).await.unwrap();

        self.discovered_workers_watch
            .send(workers.iter().map(|worker| worker.address).collect())
            .unwrap();
    }
}

struct Scheduler {
    discovered_workers: mpsc::Receiver<WorkerDescriptor>,
    workers: Arc<RwLock<HashMap<WorkerDescriptor, Worker>>>,
    frame: Arc<Frame>,

    render_tasks: mpsc::Receiver<RenderTask>,
}

impl Scheduler {
    fn new(
        discovered_workers: mpsc::Receiver<WorkerDescriptor>,
        frame: Arc<Frame>,
        render_tasks: mpsc::Receiver<RenderTask>,
    ) -> Self {
        Self {
            discovered_workers,
            workers: Arc::from(RwLock::new(HashMap::new())),
            frame,
            render_tasks,
        }
    }

    async fn schedule_render_tasks(mut self) {
        let mut discovered_workers = self.discovered_workers;
        let workers_map = self.workers.clone();
        tokio::spawn(async move {
            loop {
                let worker_descriptor = discovered_workers.recv().await.unwrap();
                let worker = Worker::connect(&worker_descriptor).await;
                workers_map
                    .write()
                    .await
                    .insert(worker_descriptor.clone(), worker);
            }
        });

        loop {
            let task = self.render_tasks.recv().await.unwrap();

            let mut workers: Vec<_> = self.workers.write().await.drain().collect();
            // TODO: Distribute render tasks.
            // TODO: Parallelize.
            for i in (0..workers.len()).rev() {
                let (_, worker) = &mut workers[i];
                if let Err(err) = worker.get_image(task.clone(), self.frame.clone()).await {
                    // TODO: Notify the worker pool user about disconnected workers.
                    println!("Error during message exchange: {}", err);
                    workers.remove(i);
                };
            }

            self.workers.write().await.extend(workers);
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct WorkerDescriptor {
    address: SocketAddr,
}

struct Worker {
    connection: WsStream,
}

impl Worker {
    async fn connect(descriptor: &WorkerDescriptor) -> Self {
        let url = format!("ws://{}", descriptor.address);
        println!("Connecting to worker {}", url);
        let connection = connect_async(url).await.unwrap().0;

        Self { connection }
    }

    async fn get_image(
        &mut self,
        render_task: RenderTask,
        frame: Arc<Frame>,
    ) -> anyhow::Result<()> {
        let render_task =
            serde_json::to_string(&render_task).expect("Failed to serialze render task");
        self.connection
            .send(Message::text(render_task))
            .await
            .context("Failed to send render task")?;

        // TODO: Process case when connection was gracefully closed.
        let image = self
            .connection
            .next()
            .await
            .unwrap()
            .context("Failed to receive image")?;
        let Message::Binary(image) = image else {
            anyhow::bail!("Unexpected message format");
        };

        let image = RenderedImage::from_bytes(image.to_vec()).image;
        frame.add_render(image).await;

        Ok(())
    }
}
