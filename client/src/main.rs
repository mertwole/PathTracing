use std::sync::Arc;

use clap::Parser;

mod frame;
mod scene;
mod window;
mod worker_pool;

use frame::Frame;

#[derive(Parser)]
pub struct Cli {}

#[tokio::main]
async fn main() {
    let _args = Cli::parse();

    let frame = Frame::new();
    let frame = Arc::from(frame);

    let worker_pool = worker_pool::start(frame.clone());

    window::start(frame, worker_pool).unwrap();
}
