use amqprs::{
    channel::{
        BasicAckArguments, BasicConsumeArguments, BasicQosArguments, Channel, QueueDeclareArguments,
    },
    connection::Connection,
    consumer::BlockingConsumer,
    BasicProperties, Deliver,
};
use clap::Parser;
use worker::scene::Scene;

use worker::api::render_task::RenderTask;
use worker::file_store::FileStore;

#[derive(Parser)]
pub struct Cli {
    #[clap(long, env = "MONGODB_URL")]
    pub mongodb_url: String,
    #[clap(long, env = "RABBITMQ_URL")]
    pub rabbitmq_url: String,
    #[clap(long, env = "RABBITMQ_QUEUE")]
    pub rabbitmq_queue: String,
}

// cargo run --release -- --mongodb-url mongodb://localhost:27017 --rabbitmq-url amqp://rmq:rmq@localhost:5672/ --rabbitmq-queue RENDER_TASKS

#[tokio::main]
async fn main() {
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
        .basic_consume_blocking(
            consumer,
            BasicConsumeArguments::new(&args.rabbitmq_queue, ""),
        )
        .await
        .unwrap();

    loop {}
}

struct RenderTaskConsumer {
    mongodb_url: String,
}

impl RenderTaskConsumer {
    pub fn new(mongodb_url: String) -> RenderTaskConsumer {
        RenderTaskConsumer { mongodb_url }
    }
}

impl BlockingConsumer for RenderTaskConsumer {
    fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let render_task_data = String::from_utf8(content).unwrap();
        let render_task: RenderTask = serde_json::de::from_str(&render_task_data).unwrap();
        println!("Processing: {}", render_task.id);

        let file_store = futures::executor::block_on(FileStore::connect(
            &self.mongodb_url,
            &render_task.scene_md5,
        ));
        let _scene = futures::executor::block_on(Scene::load(&file_store, &render_task.scene));

        let args = BasicAckArguments::new(deliver.delivery_tag(), false);
        channel.basic_ack_blocking(args).unwrap();
    }
}
