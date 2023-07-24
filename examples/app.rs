extern crate launchpad;
use launchpad::{prelude::*, Server};

mod routes;
use routes::{index, error_page, api::{data, plain}, not_found};

#[tokio::main]
async fn main() {
    Server::new()
        .router(rts!{
            routes! {
                "/" => index,
                index,
                error_page,
                data,
                plain
            },
            errors! {
                not_found
            },
        })
        .serve(3000)
        .await;
}
