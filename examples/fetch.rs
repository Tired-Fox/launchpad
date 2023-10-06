extern crate tela;

use hyper::{body::Incoming, Response};
use tela::{
    client::{fetch, SendRequest},
    Request,
};

async fn raw_fetch(url: &str) -> Response<Incoming> {
    Request::builder().uri(url).send().await
}

async fn macro_fetch(url: &str) -> Response<Incoming> {
    fetch!(url).await
}

#[tokio::main]
async fn main() {
    let url = "https://www.rust-lang.org/";

    println!("{}", raw_fetch(&url).await.status());
    println!("{}", macro_fetch(&url).await.status());
}
