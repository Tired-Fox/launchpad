//! https://hyper.rs/guides/1/client/basic/
#[cfg(feature = "macros")]
pub use tela_macros::fetch;

use std::{future::Future, pin::Pin};

use hyper::{
    body::{Body, Incoming},
    http::HeaderValue,
    Response,
};
use hyper_util::rt::TokioIo;

pub use http_body_util::{Empty, Full};
use tokio::net::TcpStream;

pub trait SendRequest {
    type Future;
    fn send(self) -> Self::Future;
}

impl<
        D: Send,
        E: Into<Box<(dyn std::error::Error + Send + Sync + 'static)>>,
        T: Body<Data = D, Error = E> + Send + 'static,
    > SendRequest for hyper::Request<T>
{
    type Future = Pin<Box<dyn Future<Output = Response<Incoming>> + Send>>;
    fn send(mut self) -> Self::Future {
        Box::pin(async move {
            let url = self.uri().clone();
            let host = url.host().expect("Fetch uri must have a host");
            let port = url.port_u16().unwrap_or(80);

            let authority = url.authority().unwrap().clone();
            let _ = self.headers_mut().insert(
                hyper::header::HOST,
                HeaderValue::from_str(authority.as_str()).unwrap(),
            );

            let address = format!("{}:{}", host, port);
            let stream = TcpStream::connect(address).await.unwrap();

            let io = TokioIo::new(stream);
            let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.unwrap();

            // Spawn a task to poll the connection, driving the HTTP state
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    println!("Connection failed: {:?}", err);
                }
            });
            let result = sender.send_request(self).await.unwrap();
            result
        })
    }
}

impl SendRequest for crate::request::Builder {
    type Future = Pin<Box<dyn Future<Output = Response<Incoming>> + Send>>;
    fn send(self) -> Self::Future {
        Box::pin(async move {
            let mut request = self.body(());
            let url = request.uri().clone();
            let host = url.host().expect("uri has no host");
            let port = url.port_u16().unwrap_or(80);

            let authority = url.authority().unwrap().clone();
            let _ = request.headers_mut().insert(
                hyper::header::HOST,
                HeaderValue::from_str(authority.as_str()).unwrap(),
            );

            let address = format!("{}:{}", host, port);
            let stream = TcpStream::connect(address).await.unwrap();

            let io = TokioIo::new(stream);
            let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.unwrap();

            // Spawn a task to poll the connection, driving the HTTP state
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    println!("Connection failed: {:?}", err);
                }
            });

            sender.send_request(request).await.unwrap().into()
        })
    }
}
