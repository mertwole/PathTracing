use futures_util::io::AsyncReadExt;

use mongodb::{
    options::{ClientOptions, GridFsBucketOptions},
    Client, GridFsBucket,
};

pub struct FileStore {
    bucket: GridFsBucket,
}

impl FileStore {
    pub async fn connect(mongodb_url: &str, scene_md5: &str) -> FileStore {
        let client_options = ClientOptions::parse(mongodb_url).await.unwrap();
        let client = Client::with_options(client_options).unwrap();

        let db = client.database("scene_files");
        let bucket = db.gridfs_bucket(Some(
            GridFsBucketOptions::builder()
                .bucket_name(scene_md5.to_string())
                .build(),
        ));

        FileStore { bucket }
    }

    pub async fn fetch_file(&self, path: &str) -> Vec<u8> {
        let mut stream = self
            .bucket
            .open_download_stream_by_name(path, None)
            .await
            .unwrap();
        let mut file_data = vec![];
        stream.read_to_end(&mut file_data).await.unwrap();
        file_data
    }
}
