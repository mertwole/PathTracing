use futures::AsyncWriteExt;
use futures_util::io::AsyncReadExt;
use image::Rgb32FImage;
use mongodb::{
    options::{ClientOptions, GridFsBucketOptions, GridFsUploadOptions},
    Client, Database,
};

use crate::api::render_task::RenderTask;

pub struct RenderStore {
    database: Database,
}

impl RenderStore {
    pub async fn connect(mongodb_url: &str) -> RenderStore {
        let client_options = ClientOptions::parse(mongodb_url).await.unwrap();
        let client = Client::with_options(client_options).unwrap();

        let database = client.database("render_outputs");

        RenderStore { database }
    }

    pub async fn save_render(&self, render_task: &RenderTask, image: Rgb32FImage) {
        let bucket = self.database.gridfs_bucket(Some(
            GridFsBucketOptions::builder()
                .bucket_name(render_task.scene_md5.to_string())
                .build(),
        ));

        let mut upload_stream = bucket.open_upload_stream(
            format!("{}", render_task.id),
            Some(GridFsUploadOptions::default()),
        );

        let raw_image = image
            .to_vec()
            .chunks(3 * image.width() as usize)
            .map(|chunk| chunk.to_vec())
            .into_iter()
            .rev()
            .flatten()
            .flat_map(f32::to_be_bytes)
            .collect::<Vec<u8>>();

        upload_stream.write_all(&raw_image).await.unwrap();
        upload_stream.close().await.unwrap();

        println!("Render {} saved", render_task.id);
    }

    pub async fn load_render(
        &self,
        id: usize,
        width: u32,
        height: u32,
        scene_md5: &str,
    ) -> Option<Rgb32FImage> {
        let bucket = self.database.gridfs_bucket(Some(
            GridFsBucketOptions::builder()
                .bucket_name(scene_md5.to_string())
                .build(),
        ));

        let mut stream = match bucket
            .open_download_stream_by_name(format!("{}", id), None)
            .await
        {
            Ok(stream) => stream,
            Err(_) => return None,
        };

        let mut render_data = vec![];
        stream.read_to_end(&mut render_data).await.unwrap();

        let render_data = render_data
            .chunks(4)
            .into_iter()
            .map(|bytes| f32::from_be_bytes(bytes.try_into().unwrap()))
            .collect();

        Some(Rgb32FImage::from_vec(width, height, render_data).unwrap())
    }
}
