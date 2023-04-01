use crate::{api::render_task::RenderTask, render_store::RenderStore, scene::Scene};

use super::Renderer;

pub struct CPURenderer<'a> {
    scene: &'a Scene,
}

#[async_trait::async_trait]
impl<'a> Renderer<'a> for CPURenderer<'a> {
    fn init(scene: &Scene) -> CPURenderer {
        CPURenderer { scene }
    }

    async fn render(&self, render_task: &RenderTask, render_store: &RenderStore) {
        // TODO: Create N threads
        // Render images
        // Save them to imagestore
        panic!()
    }
}
