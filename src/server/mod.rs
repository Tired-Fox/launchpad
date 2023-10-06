//! https://hyper.rs/guides/1/server/hello-world/

use std::net::SocketAddr;

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use self::router::IntoRouter;

pub mod router;
pub use hyper::http::StatusCode;
pub use router::Router;

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

/// Construct a tela Socket to host the server to.
///
/// # Combinations
/// - **(T1: u16)**
///     - Example: `socket!(3000)`
///     - Debug: `Socket::Local(3000)`
///     - Release: `Socket::Network(3000)`
/// - **(T1: u16, T2: u16)**
///     - Example: `socket!(3000, 4000)`
///     - Debug: `Socket::Local(3000)`
///     - Release: `Socket::Network(4000)`
/// - **(T1: Local|Network, T2: u16)**
///     - Example: `socket!(Local, 3000)`
///     - Debug: `Socket::Local(3000)`
///     - Release: `Socket::Local(3000)`
/// - **(T1: Local|Network, T2: u16, T3: u16)**
///     - Example: `socket!(Network, 3000, 4000)`
///     - Debug: `Socket::Network(3000)`
///     - Release: `Socket::Network(4000)`
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! socket {
    ($dbg_port: literal, $rls_port: literal) => {
        $crate::dbr!(
            d: $crate::server::Socket::Local($dbg_port),
            r: $crate::server::Socket::Network($rls_port)
        )
    };
    ($port: literal) => {
        $crate::dbr!(
            d: $crate::server::Socket::Local($port),
            r: $crate::server::Socket::Network($port)
        )
    };
    ($type: ident, $dbg_port: literal, $rls_port: literal) => {
        $crate::dbr!(
            d: $crate::server::Socket::$type($dbg_port),
            r: $crate::server::Socket::$type($rls_port)
        )
    };
    ($type: ident, $port: literal) => {
        $crate::dbr!(
            d: $crate::server::Socket::$type($port),
            r: $crate::server::Socket::$type($port)
        )
    };
}

#[cfg(feature = "macros")]
pub use crate::socket;

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

/// Serve a hyper + tokio async server. Let the passed in handler or Router be what each request is resolved
/// by.
///
/// See [hyper.rs v1.0 server](https://hyper.rs/guides/1/server/hello-world/) hello world examples **Starting the Server** section for a close comparison of what this method does behind the scenes.
///
/// # Example
/// ```
/// use tela::{response::Response, server::{serve, IntoSocketAddr}};
///
/// #[tokio::main]
/// async fn main() {
///     serve(Socket::default(), handler).await;
/// }
///
/// /// Request is a wrapper around hypers Request<Incoming>
/// ///
/// /// Tela chose this route as this will provide useful helpers for
/// /// operating on the current request.
/// async fn handler(_: Request) ->  Response {
///     Response::builder()
///         .body("Hello, world!")
///
///     // or
///
///     Response::builder()
///         .status(404)
///         .header("Content-Type", "text/plain")
///         .body("Could not handle request")
/// }
/// ```
pub async fn serve<Addr, R>(addr: Addr, router: R)
where
    Addr: IntoSocketAddr,
    R: IntoRouter,
{
    let addr = addr.into_socket_addr();
    let listener = TcpListener::bind(addr).await.unwrap();
    let router = router.into_router();

    #[cfg(feature = "log")]
    println!("Serving at {}", addr);

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);

        // Create owned clone of the router.
        let router = router.clone();

        tokio::task::spawn(async move {
            let result = http1::Builder::new().serve_connection(io, router).await;
            if let Err(err) = result {
                eprintln!("Error serving connection: {}", err);
            }
        });
    }
}
