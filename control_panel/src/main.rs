use amqprs::{
    callbacks::ChannelCallback,
    channel::{BasicPublishArguments, Channel, QueueDeclareArguments},
    connection::Connection,
    BasicProperties, DELIVERY_MODE_PERSISTENT,
};
use clap::Parser;
use mongodb::{options::ClientOptions, Client};

use worker::api::{render_store::RenderStore, render_task::RenderTask};

mod scene;
use scene::Scene;

#[derive(Parser)]
pub struct Cli {
    #[clap(long, env = "MONGODB_URL")]
    pub mongodb_url: String,
    #[clap(long, env = "RABBITMQ_URL")]
    pub rabbitmq_url: String,
    #[clap(long, env = "RABBITMQ_QUEUE")]
    pub rabbitmq_queue: String,
}

// Actually there will be MAX_RABBITMQ_MESSAGES not yet sent messages
// and [consumer_count] not yet ack'ed messages in queue
const MAX_RABBITMQ_MESSAGES: usize = 4;

trait BreakupRenderTask {
    fn breakup(self) -> Box<dyn Iterator<Item = RenderTask>>;
}

impl BreakupRenderTask for RenderTask {
    fn breakup(self) -> Box<dyn Iterator<Item = RenderTask>> {
        let iterations = self.config.iterations;
        Box::from(
            std::iter::repeat(self)
                .take(iterations)
                .enumerate()
                .map(|(id, mut task)| {
                    task.id = id;
                    task.config.iterations = 1;
                    task
                }),
        )
    }
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let render_task_json = &std::fs::read_to_string("./scene_data/render_task.json").unwrap();
    let mut render_task: RenderTask = serde_json::de::from_str(render_task_json).unwrap();
    let scene = Scene::load(&render_task.scene);
    // TODO: Type level safety(rendertask partially uninit on deserialize).
    render_task.scene_md5 = scene.md5.clone();

    // TODO: build kd-trees for each object.

    let client_options = ClientOptions::parse(&args.mongodb_url).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    scene.upload_to_file_store(&client).await;

    let _render_count = render_task.config.iterations;
    let _camera_res = render_task.camera.resolution;
    let _scene_md5 = render_task.scene_md5.clone();

    send_render_task(render_task, &args.rabbitmq_url, &args.rabbitmq_queue).await;

    let render_store = RenderStore::connect(&args.mongodb_url).await;
    for id in 0.._render_count {
        let res = render_store
            .load_render(id, _camera_res.x as u32, _camera_res.y as u32, &_scene_md5)
            .await;

        res.save_with_format(format!("./renders/{}.exr", id), image::ImageFormat::OpenExr)
            .unwrap();
    }
}

async fn send_render_task(render_task: RenderTask, rmq_url: &str, rmq_queue: &str) {
    let rmq_args = (rmq_url).try_into().unwrap();
    let connection = Connection::open(&rmq_args).await.unwrap();
    let channel = connection.open_channel(None).await.unwrap();
    let queue_args = QueueDeclareArguments::new(rmq_queue).durable(true).finish();
    channel.queue_declare(queue_args.clone()).await.unwrap();

    let publish_options = BasicProperties::default()
        .with_delivery_mode(DELIVERY_MODE_PERSISTENT)
        .finish();

    let mut render_tasks = render_task.breakup();
    loop {
        // Just get numer of messages in queue
        let (_, message_count, _) = channel
            .queue_declare(queue_args.clone())
            .await
            .unwrap()
            .unwrap();
        if MAX_RABBITMQ_MESSAGES > message_count as usize {
            match render_tasks.next() {
                Some(render_task) => {
                    let render_task_data = serde_json::ser::to_string(&render_task).unwrap();

                    channel
                        .basic_publish(
                            publish_options.clone(),
                            render_task_data.into_bytes(),
                            BasicPublishArguments::new("", rmq_queue),
                        )
                        .await
                        .unwrap();
                }
                None => {
                    break;
                }
            }
        }
    }

    // Debug: wait for enqueued tasks to be completed.
    loop {
        let (_, message_count, _) = channel
            .queue_declare(queue_args.clone())
            .await
            .unwrap()
            .unwrap();

        if message_count == 0 {
            break;
        }
    }

    channel.close().await.unwrap();
    connection.close().await.unwrap();
}
