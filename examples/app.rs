extern crate launchpad;
use launchpad::{prelude::*, Server};

mod routes;
use routes::{
    api::{data, plain},
    error_page, index, not_found, unexpected,
};

#[tokio::main]
async fn main() {
    Server::new()
        .router(rts! {
            [ index, error_page, data, plain ],
            catch {
                503 => unexpected,
                404 => not_found
            },
        })
        .serve(3000)
        .await;
}
