use async_trait::async_trait;
pub use tela_macros::fetch;

use hyper::{
    body::{Body, Incoming},
    http::HeaderValue,
    Response,
};
use hyper_util::rt::TokioIo;

pub use http_body_util::{Empty, Full};
use tokio::net::TcpStream;

/// When brought into scope `send()` can be called on hyper::Request and tela::Request builders to
#[async_trait]
pub trait SendRequest {
    async fn send(self) -> Response<Incoming>;
}

#[async_trait]
impl<
        D: Send,
        E: Into<Box<(dyn std::error::Error + Send + Sync + 'static)>>,
        T: Body<Data = D, Error = E> + Send + 'static,
    > SendRequest for hyper::Request<T>
{
    async fn send(mut self) -> Response<Incoming> {
        let url = self.uri().clone();
        let host = url.host().expect("Fetch uri must have a host");
        let port = url.port_u16().unwrap_or(80);

        // Hyper requires that the authority is set to send a client request
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
    }
}

#[async_trait]
impl SendRequest for crate::request::Builder {
    async fn send(self) -> Response<Incoming> {
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
    }
}
