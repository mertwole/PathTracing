use std::{net::SocketAddr, sync::Arc};

use futures::{
    SinkExt, StreamExt,
    channel::mpsc,
    stream::{SplitSink, SplitStream},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use worker::{WebSocketMessageIn, WebSocketMessageOut, api::render_task::RenderTask};

use crate::frame::Frame;

use super::WsStream;

const FILES_CHANNEL_BUFFER: usize = 8;
const RENDER_TASKS_CHANNEL_BUFFER: usize = 8;

pub struct Worker {
    render_tasks: mpsc::Sender<RenderTask>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct WorkerDescriptor {
    pub address: SocketAddr,
}

impl Worker {
    pub async fn connect(descriptor: &WorkerDescriptor, frame: Arc<Frame>) -> Self {
        let url = format!("ws://{}", descriptor.address);
        println!("Connecting to worker {}", url);
        let connection = connect_async(url).await.unwrap().0;

        let (sink, stream) = connection.split();

        let (files_sender, files_receiver) = mpsc::channel(FILES_CHANNEL_BUFFER);
        let (render_tasks_sender, render_tasks_receiver) =
            mpsc::channel(RENDER_TASKS_CHANNEL_BUFFER);

        let outbound = OutboundChannel {
            sink,
            files: files_receiver,
            render_tasks: render_tasks_receiver,
        };
        let inbound = InboundChannel {
            stream,
            frame,
            files: files_sender,
        };

        tokio::spawn(outbound.run());
        tokio::spawn(inbound.run());

        Self {
            render_tasks: render_tasks_sender,
        }
    }

    pub async fn send_render_task(&mut self, render_task: RenderTask) -> anyhow::Result<()> {
        self.render_tasks.send(render_task).await.unwrap();

        Ok(())
    }
}

struct OutboundChannel {
    sink: SplitSink<WsStream, Message>,

    files: mpsc::Receiver<(String, Vec<u8>)>,
    render_tasks: mpsc::Receiver<RenderTask>,
}

impl OutboundChannel {
    async fn run(mut self) {
        loop {
            let message = tokio::select! {
                file = self.files.recv() => {
                    let (path, content) = file.unwrap();
                    WebSocketMessageIn::File { content, path }
                },
                render_task = self.render_tasks.recv() => {
                    WebSocketMessageIn::RenderTask(Box::new(render_task.unwrap()))
                }
            };

            self.sink.send(message.serialize()).await.unwrap();
        }
    }
}

struct InboundChannel {
    stream: SplitStream<WsStream>,

    frame: Arc<Frame>,
    files: mpsc::Sender<(String, Vec<u8>)>,
}

impl InboundChannel {
    async fn run(mut self) {
        loop {
            let message = self.stream.next().await.unwrap().unwrap();
            let message = WebSocketMessageOut::deserialize(message);

            match message {
                WebSocketMessageOut::FileRequest { path } => {
                    let file_path = format!("./scene_data/{}", &path);
                    // TODO: SANITIZE!
                    let file = std::fs::read(file_path).unwrap();

                    self.files.send((path, file)).await.unwrap();
                }
                WebSocketMessageOut::Render(render) => {
                    let image = render.image;
                    self.frame.add_render(image).await;
                }
            }
        }
    }
}
