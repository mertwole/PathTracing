use std::{collections::HashMap, sync::Arc};

pub mod api;
mod camera;
mod file_store;
mod ray;
mod render_store;
mod renderer;
mod scene;

use api::render_task::RenderTask;
use file_store::FileStore;
use image::Rgb32FImage;
use renderer::{Renderer, cpu_renderer::CPURenderer};
use scene::Scene;

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
        } else {
            println!("Scene files found locally");
        }

        let scene = self.cached_scenes[&render_task.scene_md5].clone();
        let mut renderer = CPURenderer::init(scene);
        let render_task = Arc::from(render_task);

        renderer.render(render_task).await
    }
}
