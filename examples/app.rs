extern crate launchpad;
use launchpad::{prelude::*, Server};

mod routes;
use routes::{index, api::data};

#[tokio::main]
async fn main() {
    Server::new(([127, 0, 0, 1], 3000))
        .router(rts![
            index,
            data,
        ])
        .serve()
        .await;
}
