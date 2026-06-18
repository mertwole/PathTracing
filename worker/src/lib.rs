use std::{collections::HashMap, iter, sync::Arc};

use image::{EncodableLayout, Rgb32FImage};
use renderer::{Renderer, cpu_renderer::CPURenderer};
use scene::Scene;

pub mod api;
mod camera;
pub mod file_fetcher;
mod ray;
mod render_store;
mod renderer;
mod scene;

use api::render_task::RenderTask;
use serde::{Deserialize, Serialize, ser::SerializeSeq};
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

use crate::file_fetcher::FileFetcher;

pub struct Worker {
    cached_scenes: HashMap<String, Arc<Scene>>,
}

impl Worker {
    pub fn new() -> Self {
        Self {
            cached_scenes: Default::default(),
        }
    }

    pub async fn render(
        &mut self,
        render_task: RenderTask,
        file_fetcher: &impl FileFetcher,
    ) -> Rgb32FImage {
        if !self.cached_scenes.contains_key(&render_task.scene_md5) {
            println!("Loading scene files...");
            let scene = Scene::load(file_fetcher, &render_task.scene).await;
            self.cached_scenes
                .insert(render_task.scene_md5.clone(), Arc::from(scene));
            println!("Scene files loaded");
        } else {
            println!("Scene files found locally");
        }

        let scene = self.cached_scenes[&render_task.scene_md5].clone();
        let mut renderer = CPURenderer::init(scene);
        let render_task = Arc::from(render_task);

        renderer.render(render_task).await
    }
}

#[derive(Serialize, Deserialize)]
pub enum WebSocketMessageIn {
    RenderTask(Box<RenderTask>),
    File { content: Vec<u8>, path: String },
}

impl WebSocketMessageIn {
    pub fn serialize(self) -> WsMessage {
        WsMessage::binary(postcard::to_stdvec(&self).unwrap())
    }

    pub fn deserialize(message: WsMessage) -> Self {
        let WsMessage::Binary(message) = message else {
            panic!("Unexpected message format");
        };

        postcard::from_bytes(&message.to_vec()).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub enum WebSocketMessageOut {
    Render(Box<RenderedImage>),
    FileRequest { path: String },
}

impl WebSocketMessageOut {
    pub fn serialize(self) -> WsMessage {
        WsMessage::binary(postcard::to_stdvec(&self).unwrap())
    }

    pub fn deserialize(message: WsMessage) -> Self {
        let WsMessage::Binary(message) = message else {
            panic!("Unexpected message format");
        };

        postcard::from_bytes(&message.to_vec()).unwrap()
    }
}

pub struct RenderedImage {
    pub image: Rgb32FImage,
}

impl Serialize for RenderedImage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let image_bytes = self.image.as_bytes();
        let mut seq = serializer.serialize_seq(Some(2 + image_bytes.len()))?;

        seq.serialize_element(&self.image.width())?;
        seq.serialize_element(&self.image.height())?;
        seq.serialize_element(image_bytes)?;

        seq.end()
    }
}

impl<'de> Deserialize<'de> for RenderedImage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // let width = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        // let height = u32::from_le_bytes(bytes[4..8].try_into().unwrap());

        // bytes.drain(..8);

        // let data = bytes
        //     .chunks_exact(4)
        //     .map(|value| f32::from_le_bytes(value.try_into().unwrap()))
        //     .collect();

        // let image = Rgb32FImage::from_vec(width, height, data).unwrap();

        // Self { image }

        todo!()
    }
}

pub mod discovery {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Request {}

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Response {
        pub websocket_port: u16,
    }
}
