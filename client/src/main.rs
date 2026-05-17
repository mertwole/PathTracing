use std::sync::Arc;

use clap::Parser;

use worker::api::render_task::RenderTaskUninit;

mod frame;
mod scene;
mod window;
mod worker_pool;

use scene::Scene;

use crate::frame::Frame;

#[derive(Parser)]
pub struct Cli {
    #[clap(long)]
    mongodb_url: String,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let render_task_path = "./scene_data/render_task.json";
    let render_task_data = std::fs::read(render_task_path).unwrap();
    let render_task_data = String::from_utf8(render_task_data).unwrap();
    let render_task: RenderTaskUninit = serde_json::de::from_str(&render_task_data).unwrap();

    let scene = Scene::load(&render_task.scene);

    let render_task = render_task.init(scene.md5.clone());

    scene.upload_to_mongodb(&args.mongodb_url).await;

    let frame = Frame::new(
        render_task.camera.resolution.x as u32,
        render_task.camera.resolution.y as u32,
    )
    .await;
    let frame = Arc::from(frame);

    let mut worker_pool = worker_pool::WorkerPool::new();
    worker_pool.discover(30000).await;

    let frame_clone = frame.clone();
    tokio::spawn(async move {
        loop {
            worker_pool
                .send_render_task(render_task.clone(), frame_clone.clone())
                .await;
        }
    });

    window::start(frame).unwrap();
}
