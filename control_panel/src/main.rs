#[actix_web::main]
async fn main() {
    control_panel::main().await;
}
