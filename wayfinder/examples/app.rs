extern crate wayfinder;

use wayfinder::support::TokioIo;
use wayfinder::{
    prelude::*,
    request::{Body, Query},
};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{server::conn::http1, service::service_fn};
use serde::Deserialize;
use std::{convert::Infallible, error::Error, net::SocketAddr};
use tokio::net::TcpListener;

#[derive(Deserialize, Debug)]
pub struct UserQuery {
    name: String,
}

async fn handler(
    request: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, Infallible> {
    // Get all needed information from request
    let mut uri = request.uri().clone();
    let method = request.method().clone();
    // Can be used for validation, authentication, and other features
    let _headers = request.headers().clone();
    let mut body = request.collect().await.unwrap().to_bytes().to_vec();

    // Send request information to appropriate endpoint based on path and method
    match uri.path().to_string().as_str() {
        "/" if home.methods().contains(&method) => home.execute(&mut uri, &mut body),
        "/hello-world" if hello_world.methods().contains(&method) => {
            hello_world.execute(&mut uri, &mut body)
        }
        _ => Ok(hyper::Response::builder()
            .status(404)
            .body(Full::new(Bytes::from("<h1>404 Not Found</h1>")))
            .unwrap()),
    }
}

#[get("/hello-world")]
pub fn hello_world(query: Option<Query<UserQuery>>) -> String {
    match query {
        Some(Query(UserQuery { name })) => {
            format!("Hello, {}!", name)
        }
        _ => "Hello, World!".to_string(),
    }
}

#[get("/")]
pub fn home(Query(query): Query<String>, Body(body): Body<String>) -> String {
    println!("{:?}", body);
    format!("<p>query: {}</p>", query)
}

#[catch(404)]
pub fn not_found(code: u16, message: String, reason: String) -> String {
    format!("<h1>{} {}</h1>\n<p>{}</p>", code, message, reason)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    serve(([127, 0, 0, 1], 3000)).await
}

async fn serve<ADDR: Into<SocketAddr>>(addr: ADDR) -> Result<(), Box<dyn Error + Send + Sync>> {
    let addr: SocketAddr = addr.into();

    let listener = TcpListener::bind(addr.clone()).await?;
    println!("Server started at https://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handler))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
