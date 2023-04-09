use mongodb::{options::ClientOptions, Client};

pub struct ServerState {
    pub mongodb: Client,
    pub mongodb_url: String,
    pub rmq_url: String,
    pub rmq_queue: String,
}

impl ServerState {
    pub async fn new(mongodb_url: String, rmq_url: String, rmq_queue: String) -> ServerState {
        let client_options = ClientOptions::parse(&mongodb_url).await.unwrap();
        let mongodb = Client::with_options(client_options).unwrap();

        ServerState {
            mongodb,
            mongodb_url,
            rmq_url,
            rmq_queue,
        }
    }
}
