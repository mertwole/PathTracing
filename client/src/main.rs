use std::{iter, sync::Arc};

use clap::Parser;

use worker::api::render_task::RenderTaskUninit;

mod frame;
mod scene;
mod window;
mod worker_connection;

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

    let render_tasks = iter::repeat_n(render_task, 100).collect();
    tokio::spawn(worker_connection::get_images(render_tasks, frame.clone()));

    window::start().unwrap();

    //let main_window = MainWindow::init(frame).await;
    //main_window.enter_render_loop();
}
