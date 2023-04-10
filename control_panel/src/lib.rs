use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use clap::Parser;

mod rest_api;
mod scene;
mod server_state;

use server_state::ServerState;

pub mod api {
    pub use super::rest_api::{
        GetFileListResponse, GetRenderResponse, PostRenderTaskRequest, UploadFileRequest,
    };
}

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

pub async fn main() {
    let args = Cli::parse();

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
