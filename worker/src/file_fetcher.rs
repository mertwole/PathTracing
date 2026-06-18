use futures_util::io::AsyncReadExt;
use mongodb::{
    Client, GridFsBucket,
    options::{ClientOptions, GridFsBucketOptions},
};

pub trait FileFetcher {
    async fn fetch(&self, path: &str) -> Vec<u8>;
}
