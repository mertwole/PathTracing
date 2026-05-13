use std::sync::Arc;

pub mod cpu_renderer;

use image::Rgb32FImage;

use crate::{api::render_task::RenderTask, scene::Scene};

#[async_trait::async_trait]
pub trait Renderer {
    fn init(scene: Arc<Scene>) -> Self;
    async fn render(&mut self, render_task: Arc<RenderTask>) -> Rgb32FImage;
}
