extern crate launchpad;
use launchpad::{prelude::*, Server};

mod routes;
use routes::{index, error_page, api::data};

#[tokio::main]
async fn main() {
    Server::new(([127, 0, 0, 1], 3000))
        .router(rts![
            index,
            error_page,
            data,
        ])
        .serve()
        .await;
}
