use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use image::Rgb32FImage;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use worker::{RenderedImage, api::render_task::RenderTask};

use crate::frame::Frame;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn get_images(render_tasks: Vec<RenderTask>, frame: Arc<Frame>) {
    let mut connection = connect().await;

    for task in render_tasks {
        let image = get_image(&mut connection, task).await;
        frame.add_render(image).await;
    }
}

async fn connect() -> WsStream {
    let url = "ws://localhost:3000";
    connect_async(url).await.unwrap().0
}

async fn get_image(connection: &mut WsStream, render_task: RenderTask) -> Rgb32FImage {
    let render_task = serde_json::to_string(&render_task).unwrap();
    connection.send(Message::text(render_task)).await.unwrap();

    let image = connection.next().await.unwrap().unwrap();
    let Message::Binary(image) = image else {
        panic!("Wrong message format");
    };

    RenderedImage::from_bytes(image.to_vec()).image
}
