//! https://hyper.rs/guides/1/server/hello-world/

use std::net::SocketAddr;

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use self::router::IntoRouter;

pub mod error;
pub mod router;

/// Defines whether the socket address should be localhost or on the network.
pub enum Socket {
    Local(u16),
    Network(u16),
}

impl Default for Socket {
    fn default() -> Self {
        Socket::Local(3210)
    }
}

/// Convert a tuple of ([], u16) or ([u8; 4], u16) into a SocketAddr;
/// or convert a Socket into a SocketAddr.
pub trait IntoSocketAddr {
    fn into_socket_addr(self) -> SocketAddr;
}

impl IntoSocketAddr for ([u8; 4], u16) {
    fn into_socket_addr(self) -> SocketAddr {
        SocketAddr::from(self)
    }
}

impl IntoSocketAddr for ([u8; 0], u16) {
    fn into_socket_addr(self) -> SocketAddr {
        SocketAddr::from(([0, 0, 0, 0], self.1))
    }
}

impl IntoSocketAddr for Socket {
    fn into_socket_addr(self) -> SocketAddr {
        match self {
            Socket::Local(port) => SocketAddr::from(([127, 0, 0, 1], port)),
            Socket::Network(port) => SocketAddr::from(([0, 0, 0, 0], port)),
        }
    }
}

/// Serve a hyper + tokio async server. Let the passed in handler be what each request is resolved
/// by.
///
/// # Example
/// ```
/// use tela::{response, server::{serve, IntoSocketAddr, Response}};
///
/// async fn handler(req: Request<Incoming>) ->  Response {
///     Response::ok("Hello, world!")
///         .status(204)
///
///     // or
///
///     Response::error(404)
///         .header("Content-Type", "text/plain")
///         .body("Could not handle request")
/// }
/// ```
pub async fn serve<Addr, R>(
    addr: Addr,
    router: R,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    Addr: IntoSocketAddr,
    R: IntoRouter,
{
    let addr = addr.into_socket_addr();
    let listener = TcpListener::bind(addr).await?;
    let router = router.into_router();

    #[cfg(debug_assertions)]
    println!("Serving at {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let router = router.spawn();

        tokio::task::spawn(async move {
            let result = http1::Builder::new().serve_connection(io, router).await;
            if let Err(err) = result {
                eprintln!("Error serving connection: {}", err);
            }
        });
    }
}
