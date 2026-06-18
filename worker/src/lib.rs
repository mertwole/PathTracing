use std::{collections::HashMap, sync::Arc};

use image::Rgb32FImage;
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
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

#[derive(Clone, Serialize, Deserialize)]
pub struct RenderedImage {
    #[serde(serialize_with = "serialize_image")]
    #[serde(deserialize_with = "deserialize_image")]
    pub image: Rgb32FImage,
}

fn serialize_image<S: Serializer>(image: &Rgb32FImage, serializer: S) -> Result<S::Ok, S::Error> {
    let data = image
        .to_vec()
        .into_iter()
        .flat_map(f32::to_be_bytes)
        .collect();

    ImageSerde {
        width: image.width(),
        height: image.height(),
        data,
    }
    .serialize(serializer)
}

fn deserialize_image<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Rgb32FImage, D::Error> {
    let image = ImageSerde::deserialize(deserializer)?;
    let data = image
        .data
        .chunks_exact(4)
        .map(|value| f32::from_le_bytes(value.try_into().unwrap()))
        .collect();

    Ok(Rgb32FImage::from_vec(image.width, image.height, data).unwrap())
}

#[derive(Serialize, Deserialize)]
struct ImageSerde {
    width: u32,
    height: u32,
    data: Vec<u8>,
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
