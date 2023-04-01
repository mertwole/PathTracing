use crate::{api::render_task::RenderTask, render_store::RenderStore, scene::Scene};

pub mod cpu_renderer;

#[async_trait::async_trait]
pub trait Renderer<'a> {
    fn init(scene: &'a Scene) -> Self
    where
        Self: 'a;

    async fn render(&self, render_task: &RenderTask, render_store: &RenderStore);
}
