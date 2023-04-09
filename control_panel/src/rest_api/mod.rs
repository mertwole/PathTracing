use image::Rgb32FImage;
use serde::{Deserialize, Serialize};
use worker::api::{render_store::RenderStore, render_task::RenderTask};

use amqprs::{
    channel::{BasicPublishArguments, QueueDeclareArguments},
    connection::Connection,
    BasicProperties, DELIVERY_MODE_PERSISTENT,
};
use futures_util::{io::AsyncWriteExt, StreamExt};
use mongodb::{
    bson::doc,
    options::{GridFsBucketOptions, GridFsUploadOptions},
};
use std::collections::HashMap;

use crate::server_state::ServerState;
use actix_web::{web, HttpResponse};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_file).service(get_file_list);
}

#[derive(Serialize, Deserialize)]
pub struct SuccessResponse<T> {
    success: bool,
    #[serde(flatten)]
    data: T,
}

macro_rules! into_success_response {
    ($ty:ty) => {
        impl From<$ty> for actix_web::Result<HttpResponse> {
            fn from(data: $ty) -> Self {
                Ok(HttpResponse::Ok().json(data))
            }
        }
    };
}
pub(crate) use into_success_response;

#[derive(Serialize, Deserialize)]
struct PostFileRequest {
    name: String,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct PostFileResponse {}

into_success_response!(PostFileResponse);

#[actix_web::post("scene/{md5}/files")]
async fn upload_file(
    st: web::Data<ServerState>,
    md5: web::Path<String>,
    body: web::Json<PostFileRequest>,
) -> actix_web::Result<HttpResponse> {
    let mongodb_database = st.mongodb.database("scene_files");
    let bucket = mongodb_database.gridfs_bucket(Some(
        GridFsBucketOptions::builder()
            .bucket_name(md5.clone())
            .build(),
    ));

    let body = body.into_inner();

    let file_md5 = format!("{:x}", md5::compute(&body.data));

    let mut found_files = bucket
        .find(doc! { "filename": &body.name }, None)
        .await
        .expect("TODO: propagate")
        .collect::<Vec<_>>()
        .await;

    if !found_files.is_empty() {
        assert_eq!(found_files.len(), 1);

        let found_file = found_files.pop().unwrap().expect("TODO: propagate");
        let found_file_metadata = found_file
            .metadata
            .expect("Extraneous file in database: Expected metadata");
        let hash = found_file_metadata
                    .get("md5")
                    .unwrap_or_else(|| panic!(
                        "Extraneous file in database: Wrong metadata format [{}], expected md5",
                        found_file_metadata
                    ))
                    .as_str()
                    .unwrap_or_else(|| panic!(
                        "Extraneous file in database: Wrong metadata format [{}], expected md5 as string",
                        found_file_metadata
                    ));

        if hash != file_md5 {
            bucket.delete(found_file.id).await.expect("TODO: propagate");

            let mut upload_stream = bucket.open_upload_stream(
                body.name,
                Some(
                    GridFsUploadOptions::builder()
                        .metadata(Some(doc! { "md5": file_md5 }))
                        .build(),
                ),
            );

            upload_stream
                .write_all(&body.data)
                .await
                .expect("TODO: propagate");
            upload_stream.close().await.expect("TODO: propagate");
        }
    }

    PostFileResponse {}.into()
}

#[derive(Serialize, Deserialize)]
struct GetFileListResponse {
    files: HashMap<String, String>, // (path, md5)
}

into_success_response!(GetFileListResponse);

#[actix_web::get("scene/{md5}/files")]
async fn get_file_list(
    st: web::Data<ServerState>,
    md5: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let mongodb_database = st.mongodb.database("scene_files");
    let bucket = mongodb_database.gridfs_bucket(Some(
        GridFsBucketOptions::builder()
            .bucket_name(md5.clone())
            .build(),
    ));

    let files = bucket
        .find(doc! {}, None)
        .await
        .expect("TODO: propagate")
        .map(|file| {
            let file = file.expect("TODO: Propagate");

            let filename = file.filename.expect("Filename");

            let metadata = file
                .metadata
                .expect("Extraneous file in database: Expected metadata");

            let md5 = metadata
                .get("md5")
                .unwrap_or_else(|| {
                    panic!(
                        "Extraneous file in database: Wrong metadata format [{}], expected md5",
                        metadata
                    )
                })
                .as_str()
                .unwrap_or_else(|| {
                    panic!(
                "Extraneous file in database: Wrong metadata format [{}], expected md5 as string",
                metadata
                )
                })
                .to_string();

            (filename, md5)
        })
        .collect::<HashMap<_, _>>()
        .await;

    GetFileListResponse { files }.into()
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
        Box::from(std::iter::repeat(self).take(iterations).map(|mut task| {
            task.config.iterations = 1;
            task
        }))
    }
}

#[derive(Serialize, Deserialize)]
struct PostRenderTaskRequest {
    task: RenderTask,
}

#[derive(Serialize, Deserialize)]
struct PostRenderTaskResponse {}

into_success_response!(PostRenderTaskResponse);

#[actix_web::post("render_tasks")]
async fn post_render_task(
    st: web::Data<ServerState>,
    body: web::Json<PostRenderTaskRequest>,
) -> actix_web::Result<HttpResponse> {
    let rmq_args = (&*st.rmq_url).try_into().unwrap();
    let connection = Connection::open(&rmq_args).await.unwrap();
    let channel = connection.open_channel(None).await.unwrap();
    let queue_args = QueueDeclareArguments::new(&*st.rmq_queue)
        .durable(true)
        .finish();
    channel.queue_declare(queue_args.clone()).await.unwrap();

    let publish_options = BasicProperties::default()
        .with_delivery_mode(DELIVERY_MODE_PERSISTENT)
        .finish();

    let mut render_tasks = body.into_inner().task.breakup();
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
                            BasicPublishArguments::new("", &st.rmq_queue),
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

    channel.close().await.unwrap();
    connection.close().await.unwrap();

    PostRenderTaskResponse {}.into()
}

#[derive(Serialize, Deserialize)]
struct GetRenderResponse {
    image_data: Vec<f32>,
    image_width: u32,
    image_height: u32,
}

into_success_response!(GetRenderResponse);

#[derive(Serialize, Deserialize)]
struct GetRenderPath {
    render_task_md5: String,
}

#[actix_web::get("render_tasks/{render_task_md5}/render")]
async fn get_render(
    st: web::Data<ServerState>,
    path: web::Path<GetRenderPath>,
) -> actix_web::Result<HttpResponse> {
    let render_store = RenderStore::connect(&st.mongodb_url).await;
    let render_count = render_store.render_count(&path.render_task_md5).await;

    if render_count == 0 {
        return GetRenderResponse {
            image_data: vec![],
            image_width: 0,
            image_height: 0,
        }
        .into();
    }

    let mut res = None;
    let multiplier = 1.0 / (render_count as f32);
    for i in 0..render_count {
        let render = render_store
            .load_render(i, &path.render_task_md5)
            .await
            .unwrap();

        if res.is_none() {
            res = Some(Rgb32FImage::new(render.width(), render.height()));
        }

        if let Some(res) = res.as_mut() {
            for x in 0..res.width() {
                for y in 0..res.height() {
                    for i in 0..3 {
                        res.get_pixel_mut(x, y).0[i] += render.get_pixel(x, y).0[i] * multiplier;
                    }
                }
            }
        }
    }

    let res = res.unwrap();

    GetRenderResponse {
        image_width: res.width(),
        image_height: res.height(),
        image_data: res.into_raw(),
    }
    .into()
}
