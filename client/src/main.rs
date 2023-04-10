use clap::Parser;
use control_panel::api::{GetRenderResponse, PostRenderTaskRequest};
use image::Rgba32FImage;
use scene::Scene;
use worker::api::render_task::{RenderTask, RenderTaskUninit};

mod scene;

#[derive(Parser)]
pub struct Cli {
    #[clap(long, env = "CONTROL_PANEL_URL")]
    pub control_panel_url: String,
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
    let render_task_md5 = render_task.md5();

    scene
        .upload_to_control_panel(&args.control_panel_url, &render_task_md5)
        .await;

    send_render_task(&args.control_panel_url, render_task).await;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        save_render(&args.control_panel_url, &render_task_md5).await;
    }
}

async fn send_render_task(control_panel_url: &str, render_task: RenderTask) {
    let client = reqwest::Client::new();

    let body = PostRenderTaskRequest { task: render_task };

    client
        .post(format!("{}/render_tasks", control_panel_url))
        .json(&body)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

async fn save_render(control_panel_url: &str, render_task_md5: &str) {
    let res = reqwest::get(format!(
        "{}/render_tasks/{}/render",
        control_panel_url, render_task_md5
    ))
    .await
    .unwrap()
    .error_for_status()
    .unwrap();
    let res: GetRenderResponse = res.json().await.unwrap();

    if res.image_data.len() == 0 {
        return;
    }

    let image = Rgba32FImage::from_raw(res.image_width, res.image_height, res.image_data).unwrap();

    image
        .save_with_format("./renders/0.exr", image::ImageFormat::OpenExr)
        .unwrap();
}
