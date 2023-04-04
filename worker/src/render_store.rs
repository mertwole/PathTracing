use image::RgbaImage;

use crate::api::render_task::RenderTask;

pub struct RenderStore {}

impl RenderStore {
    pub fn new() -> RenderStore {
        RenderStore {}
    }

    pub async fn save_render(&self, render_task: &RenderTask, image: RgbaImage) {
        todo!()
    }
}
