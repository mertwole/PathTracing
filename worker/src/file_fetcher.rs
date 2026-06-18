pub trait FileFetcher {
    async fn fetch(&self, path: &str) -> Vec<u8>;
}
