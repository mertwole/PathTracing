use futures::StreamExt;
use image::Rgb32FImage;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::connect_async;
use worker::{RenderedImage, api::render_task::RenderTask};

#[derive(Clone)]
struct ClientState {
    render_task: RenderTask,
    sender: Sender<Rgb32FImage>,
}

pub async fn get_image(render_task: RenderTask) -> Rgb32FImage {
    let url = "ws://localhost:3000";

    let (stream, _) = connect_async(url).await.unwrap();
    let (stream_tx, stream_rx) = stream.split();

    // TODO: Send render task and receive image.

    todo!()
}
