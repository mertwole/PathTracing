#[tokio::main]
async fn main() {
    worker::startup().await;
}
