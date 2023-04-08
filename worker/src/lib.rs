use std::{collections::HashMap, sync::Arc, time::Duration};

use amqprs::{
    channel::{
        BasicAckArguments, BasicConsumeArguments, BasicQosArguments, Channel, QueueDeclareArguments,
    },
    connection::Connection,
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};
use clap::Parser;

pub mod api;
mod camera;
mod file_store;
mod ray;
mod render_store;
mod renderer;
mod scene;

use api::render_task::RenderTask;
use file_store::FileStore;
use render_store::RenderStore;
use renderer::{cpu_renderer::CPURenderer, Renderer};
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

pub async fn startup() {
    let args = Cli::parse();

    let rmq_args = (&*args.rabbitmq_url).try_into().unwrap();
    let connection = Connection::open(&rmq_args).await.unwrap();
    let channel = connection.open_channel(None).await.unwrap();
    let queue_args = QueueDeclareArguments::new(&args.rabbitmq_queue)
        .durable(true)
        .finish();
    channel.queue_declare(queue_args).await.unwrap();
    channel
        .basic_qos(
            BasicQosArguments::new(0, 1, false)
                .prefetch_count(1)
                .finish(),
        )
        .await
        .unwrap();

    let consumer = RenderTaskConsumer::new(args.mongodb_url);
    channel
        .basic_consume(
            consumer,
            BasicConsumeArguments::new(&args.rabbitmq_queue, ""),
        )
        .await
        .unwrap();

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

struct RenderTaskConsumer {
    mongodb_url: String,
    cached_scenes: HashMap<String, Arc<Scene>>,
}

impl RenderTaskConsumer {
    pub fn new(mongodb_url: String) -> RenderTaskConsumer {
        RenderTaskConsumer {
            mongodb_url,
            cached_scenes: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl AsyncConsumer for RenderTaskConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let render_task_data = String::from_utf8(content).unwrap();
        let render_task: RenderTask = serde_json::de::from_str(&render_task_data).unwrap();

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
        let render_store = RenderStore::connect(&self.mongodb_url).await;
        let render_task = Arc::from(render_task);
        renderer.render(render_task, &render_store).await;

        let args = BasicAckArguments::new(deliver.delivery_tag(), false);
        channel.basic_ack(args).await.unwrap();
    }
}
