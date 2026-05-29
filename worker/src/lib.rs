use std::{collections::HashMap, iter, sync::Arc};

use file_store::FileStore;
use image::Rgb32FImage;
use renderer::{Renderer, cpu_renderer::CPURenderer};
use scene::Scene;

pub mod api;
mod camera;
mod file_store;
mod ray;
mod render_store;
mod renderer;
mod scene;

use api::render_task::RenderTask;

pub struct Worker {
    mongodb_url: String,
    cached_scenes: HashMap<String, Arc<Scene>>,
}

impl Worker {
    pub fn new(mongodb_url: String) -> Self {
        Self {
            mongodb_url,
            cached_scenes: Default::default(),
        }
    }

    pub async fn render(&mut self, render_task: RenderTask) -> Rgb32FImage {
        if !self.cached_scenes.contains_key(&render_task.scene_md5) {
            println!("Loading scene files...");
            let file_store = FileStore::connect(&self.mongodb_url, &render_task.scene_md5).await;
            let scene = Scene::load(&file_store, &render_task.scene).await;
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

pub struct RenderedImage {
    pub image: Rgb32FImage,
}

impl RenderedImage {
    pub fn to_bytes(self) -> Vec<u8> {
        iter::once(self.image.width().to_le_bytes())
            .chain(iter::once(self.image.height().to_le_bytes()))
            .chain(self.image.iter().map(|value| value.to_le_bytes()))
            .flatten()
            .collect()
    }

    pub fn from_bytes(mut bytes: Vec<u8>) -> Self {
        let width = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        let height = u32::from_le_bytes(bytes[4..8].try_into().unwrap());

        bytes.drain(..8);

        let data = bytes
            .chunks_exact(4)
            .map(|value| f32::from_le_bytes(value.try_into().unwrap()))
            .collect();

        let image = Rgb32FImage::from_vec(width, height, data).unwrap();

        Self { image }
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
