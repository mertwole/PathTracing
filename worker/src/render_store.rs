use futures::AsyncWriteExt;
use image::{ImageFormat, RgbaImage};
use itertools::Itertools;

use crate::api::render_task::RenderTask;

use mongodb::{
    options::{ClientOptions, GridFsBucketOptions, GridFsUploadOptions},
    Client, Database,
};

pub struct RenderStore {
    database: Database,
}

impl RenderStore {
    pub async fn connect(mongodb_url: &str) -> RenderStore {
        let client_options = ClientOptions::parse(mongodb_url).await.unwrap();
        let client = Client::with_options(client_options).unwrap();

        let database = client.database("scene_files");

        RenderStore { database }
    }

    // TODO: Save in correct format (not as raw bytes)
    pub async fn save_render(&self, render_task: &RenderTask, image: RgbaImage) {
        let bucket = self.database.gridfs_bucket(Some(
            GridFsBucketOptions::builder()
                .bucket_name(render_task.scene_md5.to_string())
                .build(),
        ));

        let mut upload_stream = bucket.open_upload_stream(
            format!("{}", render_task.id),
            Some(GridFsUploadOptions::default()),
        );

        let width = image.width();
        let height = image.height();
        let image_data = image.to_vec();

        let raw_image = image_data
            .chunks(4 * width as usize)
            .map(|chunk| chunk.to_vec())
            .into_iter()
            .rev()
            .flatten()
            .collect::<Vec<u8>>();

        image::save_buffer_with_format(
            format!("./debug_output/{}.bmp", render_task.id),
            &raw_image,
            width,
            height,
            image::ColorType::Rgba8,
            image::ImageFormat::Bmp,
        )
        .unwrap();

        println!("Render saved!");

        upload_stream.write_all(&image_data).await.unwrap();
    }
}
