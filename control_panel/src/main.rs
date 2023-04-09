use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use amqprs::{
    channel::{BasicPublishArguments, QueueDeclareArguments},
    connection::Connection,
    BasicProperties, DELIVERY_MODE_PERSISTENT,
};
use clap::Parser;
use image::Rgb32FImage;
use math::UVec2;
use mongodb::{options::ClientOptions, Client};

mod rest_api;
mod scene;
mod server_state;

use scene::Scene;
use server_state::ServerState;
use worker::api::{render_store::RenderStore, render_task::RenderTask};

#[derive(Parser)]
pub struct Cli {
    #[clap(long, env = "MONGODB_URL")]
    pub mongodb_url: String,
    #[clap(long, env = "RABBITMQ_URL")]
    pub rabbitmq_url: String,
    #[clap(long, env = "RABBITMQ_QUEUE")]
    pub rabbitmq_queue: String,
    #[clap(long, env = "APP_ENDPOINT")]
    pub app_endpoint: String,
}

#[actix_web::main]
async fn main() {
    let args = Cli::parse();

    // -------------------------
    // let render_task_json = &std::fs::read_to_string("./scene_data/render_task.json").unwrap();
    // let mut render_task: RenderTask = serde_json::de::from_str(render_task_json).unwrap();
    // let scene = Scene::load(&render_task.scene);
    // // TODO: Type level safety(rendertask partially uninit on deserialize).
    // render_task.scene_md5 = scene.md5.clone();

    // // TODO: build kd-trees for each object.

    // let client_options = ClientOptions::parse(&args.mongodb_url).await.unwrap();
    // let client = Client::with_options(client_options).unwrap();
    // scene.upload_to_file_store(&client).await;

    // let _render_count = render_task.config.iterations;
    // let _camera_res = render_task.camera.resolution;
    // let _scene_md5 = render_task.scene_md5.clone();

    // send_render_task(render_task, &args.rabbitmq_url, &args.rabbitmq_queue).await;

    // save_renders(&args.mongodb_url, _camera_res, &_scene_md5, _render_count).await;
    // -------------------------

    let server_state =
        Arc::new(ServerState::new(args.mongodb_url, args.rabbitmq_url, args.rabbitmq_queue).await);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(Arc::clone(&server_state)))
            .configure(rest_api::config)
    })
    .bind(args.app_endpoint)
    .unwrap()
    .run()
    .await
    .unwrap();
}

async fn save_renders(
    mongodb_url: &str,
    camera_resolution: UVec2,
    scene_md5: &str,
    iterations: usize,
) {
    // let render_store = RenderStore::connect(mongodb_url).await;
    // let mut renders: Vec<_> = std::iter::repeat(None).take(iterations).collect();

    // let mut id = 0;
    // while id < iterations {
    //     // NOT SCENE_MD5
    //     // let res = render_store
    //     //     .load_render(
    //     //         id,
    //     //         camera_resolution.x as u32,
    //     //         camera_resolution.y as u32,
    //     //         scene_md5,
    //     //     )
    //     //     .await;

    //     // if res.is_none() {
    //     //     continue;
    //     // }

    //     // renders[id] = res;
    //     // id += 1;
    // }

    // let renders: Vec<_> = renders.into_iter().flatten().collect();

    // let mut res = Rgb32FImage::new(camera_resolution.x as u32, camera_resolution.y as u32);

    // let multiplier = 1.0 / (iterations as f32);
    // for render in renders {
    //     for x in 0..camera_resolution.x as u32 {
    //         for y in 0..camera_resolution.y as u32 {
    //             for i in 0..3 {
    //                 res.get_pixel_mut(x, y).0[i] += render.get_pixel(x, y).0[i] * multiplier;
    //             }
    //         }
    //     }
    // }

    // res.save_with_format("./renders/output.exr", image::ImageFormat::OpenExr)
    //     .unwrap();
}

async fn send_render_task(render_task: RenderTask, rmq_url: &str, rmq_queue: &str) {
    // let rmq_args = (rmq_url).try_into().unwrap();
    // let connection = Connection::open(&rmq_args).await.unwrap();
    // let channel = connection.open_channel(None).await.unwrap();
    // let queue_args = QueueDeclareArguments::new(rmq_queue).durable(true).finish();
    // channel.queue_declare(queue_args.clone()).await.unwrap();

    // let publish_options = BasicProperties::default()
    //     .with_delivery_mode(DELIVERY_MODE_PERSISTENT)
    //     .finish();

    // let mut render_tasks = render_task.breakup();
    // loop {
    //     // Just get numer of messages in queue
    //     let (_, message_count, _) = channel
    //         .queue_declare(queue_args.clone())
    //         .await
    //         .unwrap()
    //         .unwrap();
    //     if MAX_RABBITMQ_MESSAGES > message_count as usize {
    //         match render_tasks.next() {
    //             Some(render_task) => {
    //                 let render_task_data = serde_json::ser::to_string(&render_task).unwrap();

    //                 channel
    //                     .basic_publish(
    //                         publish_options.clone(),
    //                         render_task_data.into_bytes(),
    //                         BasicPublishArguments::new("", rmq_queue),
    //                     )
    //                     .await
    //                     .unwrap();
    //             }
    //             None => {
    //                 break;
    //             }
    //         }
    //     }
    // }

    // channel.close().await.unwrap();
    // connection.close().await.unwrap();
}
