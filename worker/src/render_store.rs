use std::io::BufWriter;

use futures_util::{io::AsyncWriteExt, AsyncReadExt, StreamExt};
use image::{
    codecs::openexr::{OpenExrDecoder, OpenExrEncoder},
    Rgb32FImage,
};
use mongodb::{
    bson::doc,
    options::ClientOptions,
    options::{GridFsBucketOptions, GridFsUploadOptions},
    Client, Database, GridFsBucket,
};

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

    pub async fn save_render(&self, render_task_md5: String, image: Rgb32FImage) {
        let bucket = self.database.gridfs_bucket(Some(
            GridFsBucketOptions::builder()
                .bucket_name(render_task_md5)
                .build(),
        ));

        let render_count = Self::render_count_internal(&bucket).await;

        let mut upload_stream = bucket.open_upload_stream(
            format!("{}", render_count),
            Some(
                GridFsUploadOptions::builder()
                    .metadata(Some(doc! {
                        "width": image.width(),
                        "height": image.height()
                    }))
                    .build(),
            ),
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

        println!("Render {} saved", render_count);
    }

    async fn render_count_internal(bucket: &GridFsBucket) -> usize {
        bucket
            .find(doc! {}, None)
            .await
            .expect("TODO: propagate")
            .collect::<Vec<_>>()
            .await
            .len()
    }

    pub async fn render_count(&self, render_task_md5: &str) -> usize {
        let bucket = self.database.gridfs_bucket(Some(
            GridFsBucketOptions::builder()
                .bucket_name(render_task_md5.to_string())
                .build(),
        ));

        Self::render_count_internal(&bucket).await
    }

    pub async fn load_render(&self, id: usize, render_task_md5: &str) -> Option<Rgb32FImage> {
        let bucket = self.database.gridfs_bucket(Some(
            GridFsBucketOptions::builder()
                .bucket_name(render_task_md5.to_string())
                .build(),
        ));

        let mut found_files = bucket
            .find(doc! { "filename": id.to_string() }, None)
            .await
            .expect("TODO: propagate")
            .collect::<Vec<_>>()
            .await;
        assert_eq!(found_files.len(), 1);
        let found_file = found_files.pop().unwrap().expect("TODO: propagate");
        let found_file_metadata = found_file
            .metadata
            .expect("Extraneous file in database: Expected metadata");
        let width = found_file_metadata
            .get("width")
            .unwrap_or_else(|| {
                panic!(
                    "Extraneous file in database: Wrong metadata format [{}], expected width",
                    found_file_metadata
                )
            })
            .as_i64()
            .unwrap_or_else(|| {
                panic!(
                "Extraneous file in database: Wrong metadata format [{}], expected width as integer",
                found_file_metadata
            )
            }) as u32;
        let height = found_file_metadata
            .get("height")
            .unwrap_or_else(|| {
                panic!(
                    "Extraneous file in database: Wrong metadata format [{}], expected height",
                    found_file_metadata
                )
            })
            .as_i64()
            .unwrap_or_else(|| {
                panic!(
                "Extraneous file in database: Wrong metadata format [{}], expected height as integer",
                found_file_metadata
            )
            }) as u32;

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
