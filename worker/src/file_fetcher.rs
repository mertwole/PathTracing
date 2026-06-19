pub trait FileFetcher {
    fn fetch(&self, path: &str) -> impl Future<Output = Vec<u8>>;
}
