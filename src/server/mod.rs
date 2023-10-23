//! https://hyper.rs/guides/1/server/hello-world/

use std::{net::SocketAddr, sync::Arc};

use hyper::{body::Incoming, server::conn::http1};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use self::router::{route::Captures, IntoRouter};

pub mod router;
pub use hyper::http::StatusCode;
pub use router::Router;

/// Represents a local or network socket.
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

use crate::cookie::CookieJar;

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

#[derive(Default)]
pub struct State {
    cookies: CookieJar,
    catches: Captures,
}

impl State {
    pub fn new(request: &hyper::Request<Incoming>, catches: Captures) -> Self {
        State {
            cookies: CookieJar::new(match request.headers().get(hyper::header::COOKIE) {
                Some(v) => v.to_str().unwrap().to_string(),
                None => String::new(),
            }),
            catches,
        }
    }

    pub fn cookies(&self) -> &CookieJar {
        &self.cookies
    }
    pub fn cookies_mut(&mut self) -> &mut CookieJar {
        &mut self.cookies
    }
}

/// Server struct that handles some events around binding and connections.
///
/// The `serve` method when called will start a server and handle request and put them through
/// the provided router.
pub struct Server {
    on_bind: Option<Box<dyn Fn(SocketAddr)>>,
    on_connection: Option<Box<dyn Fn(SocketAddr)>>,
    on_connection_error: Option<Arc<dyn Fn(Box<dyn std::error::Error>) + Send + Sync>>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            on_bind: None,
            on_connection: None,
            on_connection_error: None,
        }
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Serve a hyper + tokio async server. Let the passed in handler or Router be what each request is resolved
    /// by.
    ///
    /// See [hyper.rs v1.0 server](https://hyper.rs/guides/1/server/hello-world/) hello world example **Starting the Server** section for a close comparison of what this method does behind the scenes.
    ///
    /// # Example
    /// ```
    /// use tela::{server::{Server, IntoSocketAddr}};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     Server::new().serve(
    ///         Socket::default(),
    ///         handler
    ///     ).await
    /// }
    ///
    /// async fn handler() {}
    /// ```
    pub async fn serve<Addr, R>(self, addr: Addr, router: R)
    where
        Addr: IntoSocketAddr,
        R: IntoRouter,
    {
        let addr = addr.into_socket_addr();
        let listener = TcpListener::bind(addr).await.unwrap();
        let router = router.into_router();

        if let Some(on_bind) = &self.on_bind {
            on_bind(listener.local_addr().unwrap())
        }

        loop {
            let (stream, addr) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);

            if let Some(on_connection) = &self.on_connection {
                on_connection(addr)
            }

            // Create owned clone of the router.
            let router = router.clone();
            let error_handler = self.on_connection_error.clone();

            tokio::task::spawn(async move {
                let result = http1::Builder::new().serve_connection(io, router).await;
                if let Err(err) = result {
                    if let Some(on_connection_error) = error_handler {
                        on_connection_error(err.into())
                    }
                }
            });
        }
    }
}

pub struct Builder(Server);
impl Builder {
    pub fn new() -> Builder {
        Builder(Server::new())
    }

    /// Define a handler that is called with a `SocketAddr` when the server finishes binding.
    pub fn on_bind<F>(mut self, handler: F) -> Self
    where
        F: Fn(SocketAddr) + 'static,
    {
        self.0.on_bind = Some(Box::new(handler));
        self
    }

    /// Define a handler that is called for every request connection.
    pub fn on_connection<F>(mut self, handler: F) -> Self
    where
        F: Fn(SocketAddr) + 'static,
    {
        self.0.on_connection = Some(Box::new(handler));
        self
    }

    /// Define a handler that is called for every unhandled server hyper error.
    pub fn on_connection_error<F>(mut self, handler: F) -> Self
    where
        F: Fn(Box<dyn std::error::Error>) + Send + Sync + 'static,
    {
        self.0.on_connection_error = Some(Arc::new(handler));
        self
    }

    pub fn build(self) -> Server {
        self.0
    }

    /// Serve a hyper + tokio async server. Let the passed in handler or Router be what each request is resolved
    /// by.
    ///
    /// See [hyper.rs v1.0 server](https://hyper.rs/guides/1/server/hello-world/) hello world example **Starting the Server** section for a close comparison of what this method does behind the scenes.
    ///
    /// # Example
    /// ```
    /// use tela::{server::{Server, IntoSocketAddr}};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     Server::new().serve(
    ///         Socket::default(),
    ///         handler
    ///     ).await
    /// }
    ///
    /// async fn handler() {}
    /// ```
    pub async fn serve<Addr, R>(self, addr: Addr, router: R)
    where
        Addr: IntoSocketAddr,
        R: IntoRouter,
    {
        self.build().serve(addr, router).await;
    }
}
