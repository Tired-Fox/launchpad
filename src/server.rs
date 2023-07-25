pub use hyper::Method;
use hyper::{server::conn::http1, service::service_fn};
use std::net::SocketAddr;

use std::sync::{Arc, Mutex};
use tokio::{
    net::TcpListener,
    sync::mpsc::{self, Sender},
};

use super::support::TokioIo;
use launchpad_router::{Command, RouteHandler, Router};

/// Async server object that handles requests
///
/// The server will communicate with a router thread to serve requests
///
/// # Example
/// ```
/// use launchpad::{prelude::*, Server};
///
/// fn main() {
///     Server::new(([127, 0, 0, 1], 3000))
///         .router(routes![home])
///         .serve()
///         .await;
/// }
///
/// #[get("/")]
/// fn home() -> Result<&'static str> {
///     Ok("Hello, world!")
/// }
/// ```
pub struct Server {
    router: Arc<Mutex<Router>>,
}

impl Server {
    /// Create a new server
    ///
    /// The server can then be served with the `serve` method with a given address
    /// which can be anything that can be converted into a `SocketAddr`.
    ///
    /// # Example
    /// ```rust
    /// use launchpad::{prelude::*, Server};
    ///
    /// fn main() {
    ///     Server::new()
    ///         .serve(([127, 0, 0, 1], 3000))
    ///         .await;
    /// }
    /// ```
    ///
    /// ```rust
    /// use launchpad::{prelude::*, Server};
    ///
    /// fn main() {
    ///     Server::new()
    ///         .serve(3000)
    ///         .await;
    /// }
    /// ```
    pub fn new() -> Self {
        Server {
            router: Arc::new(Mutex::new(Router::new())),
        }
    }

    /// Start listener thread for handling access to router
    ///
    /// Creates mpsc channel and returns Sender handle. The thread that this method
    /// creates is the only instance of the router that should exists.
    fn serve_routes(&self) -> Sender<Command> {
        let (tx, mut rx) = mpsc::channel::<Command>(32);
        let router = self.router.clone();

        tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                use Command::*;

                match cmd {
                    Get {
                        method,
                        path,
                        response,
                    } => {
                        let router = router.lock().unwrap();
                        response
                            .send(router.get_route(method, path).map(|f| f.clone()))
                            .unwrap();
                    }
                    Error {
                        code,
                        reason,
                        response,
                    } => {
                        let router = router.lock().unwrap();
                        response.send(router.get_error(code, reason)).unwrap()
                    }
                }
            }
        });

        tx
    }

    /// Prints the cli banner for when the server starts
    fn cli_banner(&self, addr: &SocketAddr) {
        let message = "http://";
        let fill = (0..addr.to_string().len() + message.len() + 16)
            .map(|_| '╌')
            .collect::<String>();
        println!(
            "{}",
            format!(
                "
╭{}╮
╎ 🚀 \x1b[33;1mLaunchpad\x1b[39;22m: {}{} ╎
╰{}╯
",
                fill, message, addr, fill
            )
        );
    }

    /// Starts the server and handles requests
    ///
    /// # Example
    /// ```rust
    /// use launchpad::{prelude::*, Server};
    ///
    /// fn main() {
    ///     Server::new()
    ///         .serve(3000)
    ///         .await;
    /// }
    /// ```
    pub async fn serve<Socket: IntoSocket>(&self, addr: Socket) {
        let addr: SocketAddr = addr.into_socket();

        // Start server
        let listener = TcpListener::bind(addr.clone()).await.unwrap();
        // Start router channel/listener and get a handle to it
        let tx = self.serve_routes();

        // CLI log to indicate server is up

        #[cfg(debug_assertions)]
        self.cli_banner(&addr);
        #[cfg(not(debug_assertions))]
        println!("{}", addr);

        // Create a single route handler/resolver that has a handle to the router channel
        let handler = Arc::new(RouteHandler::new(tx.clone()));

        // Loop through the connections an resolve them with async tasks
        loop {
            let (stream, _) = listener.accept().await.unwrap();

            // PERF: This is currently only because hyper read and write needs to be
            // impl for new tokio read and write streams. Caused by newest version of
            // tokio.
            let io = TokioIo::new(stream);

            // Get new pointer to RouteHandler
            let rh = handler.clone();

            // Spawn task to handle reqeust
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(|req| rh.parse(req)))
                    .await
                {
                    eprintln!("Failed to serve connection: {:?}", err);
                }
            });
        }
    }

    /// Set the router for the server
    ///
    /// The router object holds all information for url to endpoint mappings
    /// along with custom error responses.
    pub fn router(self, router: Router) -> Self {
        Server {
            router: Arc::new(Mutex::new(router)),
            ..self
        }
    }
}

pub trait IntoSocket {
    fn into_socket(self) -> SocketAddr;
}

impl IntoSocket for ([u8; 4], u16) {
    fn into_socket(self) -> SocketAddr {
        SocketAddr::from(self)
    }
}

impl IntoSocket for u16 {
    fn into_socket(self) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], self))
    }
}
