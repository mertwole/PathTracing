use futures::{SinkExt, StreamExt};
use image::Rgb32FImage;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use worker::{RenderedImage, api::render_task::RenderTask};

pub async fn get_image(render_task: RenderTask) -> Rgb32FImage {
    let url = "ws://localhost:3000";

    let (stream, _) = connect_async(url).await.unwrap();
    let (mut stream_tx, mut stream_rx) = stream.split();

    let render_task = serde_json::to_string(&render_task).unwrap();
    stream_tx.send(Message::text(render_task)).await.unwrap();

    let image = stream_rx.next().await.unwrap().unwrap();
    let Message::Binary(image) = image else {
        panic!("Wrong message format");
    };

    RenderedImage::from_bytes(image.to_vec()).image
}
