use crate::response::template::TemplateEngine;
use std::{error::Error, fmt::Display, net::SocketAddr, sync::Arc};

use cfg_if::cfg_if;
use hyper::{server::conn::http1, service::service_fn};
use tokio::net::TcpListener;

use crate::{
    prelude::{Catch, Endpoint},
    support::TokioIo,
    Router,
};

pub trait IntoSocketAddr {
    fn into_socket_addr(self) -> SocketAddr;
}

impl IntoSocketAddr for u16 {
    fn into_socket_addr(self) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], self))
    }
}

impl IntoSocketAddr for ([u8; 4], u16) {
    fn into_socket_addr(self) -> SocketAddr {
        SocketAddr::from(self)
    }
}

/// Contains a router and handles setting up:
/// * routes
/// * error handlers
/// * static asset path
/// * tera asset path
///
/// Serves requests from the given port based on uri path and request method.
///
/// # Example
/// ```
/// use wayfinder::{prelude::*, Server};
///
/// #[get("/")]
/// fn home() -> HTML<String> {
///     html!(<h1>"Hello, world"!</h1>)
/// }
///
/// #[wayfinder::main]
/// async fn main() {
///     Server::new()
///         .route(home)
///         .serve(3000)
///         .await
/// }
/// ```
pub struct Server {
    router: Router,
}

#[cfg(feature = "handlebars")]
impl Server {
    /// Setup the tera template root path
    ///
    /// This exposes all files in that path to the tera templating
    /// engine.
    pub fn handlebars<T: Into<String>>(
        self,
        path: T,
        globals: std::collections::BTreeMap<String, serde_json::Value>,
    ) -> Self {
        crate::response::template::Handlebars::init(path, globals);
        self
    }
}

#[cfg(feature = "tera")]
impl Server {
    /// Setup the tera template root path
    ///
    /// This exposes all files in that path to the tera templating
    /// engine.
    pub fn tera<T: Into<String>>(
        self,
        path: T,
        globals: std::collections::BTreeMap<String, serde_json::Value>,
    ) -> Self {
        crate::response::template::Tera::init(path, globals);
        self
    }
}

impl Server {
    pub fn new() -> Self {
        Server {
            router: Router::new(),
        }
    }

    /// Set where static files should be served from
    pub fn assets<T: Display>(mut self, path: T) -> Self {
        self.router.assets(path.to_string());
        self
    }

    /// Add a route to the router
    ///
    /// Must have `impl Endpoint`.
    /// Wrap a method with a request macro; ex: `#[get('/')]`.
    ///
    /// # Example
    /// ```
    /// use wayfinder::prelude::*;
    /// #[get("/")]
    /// fn home() -> String { ... }
    ///
    /// async main() {
    ///     Server::new()
    ///         .route(home)
    ///         .serve(3000)
    ///         .await
    /// }
    /// ```
    pub fn route<T: Endpoint + 'static>(mut self, route: T) -> Self {
        self.router.route(Arc::new(route));
        self
    }

    /// List of routes to add to the router
    ///
    /// Must be an array with each item being Arc<dyn Endpoint>.
    /// Use the `group![]` macro to automatically wrap each `impl Endpoint`
    /// with `Arc::new()`
    ///
    /// # Example
    /// ```
    /// use wayfinder::prelude::*;
    /// #[get("/")]
    /// fn home() -> String { ... }
    /// #[get("/blog")]
    /// fn blog() -> String { ... }
    ///
    /// async main() {
    ///     Server::new()
    ///         .routes(group![home, blog])
    ///         .serve(3000)
    ///         .await
    /// }
    /// ```
    pub fn routes<const SIZE: usize>(mut self, routes: [Arc<dyn Endpoint>; SIZE]) -> Self {
        for route in routes {
            self.router.route(route);
        }
        self
    }

    /// Add a error handler to the router
    ///
    /// Must have `impl Catch`.
    /// Wrap a method with a catch macro; ex: `#[catch(404)]`.
    ///
    /// # Example
    /// ```
    /// use wayfinder::prelude::*;
    /// #[catch(404)]
    /// fn not_found(...) -> String { ... }
    ///
    /// async main() {
    ///     Server::new()
    ///         .catch(not_found]
    ///         .serve(3000)
    ///         .await
    /// }
    /// ```
    pub fn catch<T: Catch + 'static>(mut self, catch: T) -> Self {
        self.router.catch(Arc::new(catch));
        self
    }

    /// List of error handlers to add to the router
    ///
    /// Must be an array with each item being Arc<dyn Catch>.
    /// Use the `group![]` macro to automatically wrap each `impl Catch`
    /// with `Arc::new()`
    ///
    /// # Example
    /// ```
    /// use wayfinder::prelude::*;
    /// #[catch(404)]
    /// fn not_found(...) -> String { ... }
    /// #[catch(500)]
    /// fn internal_server(...) -> String { ... }
    ///
    /// async main() {
    ///     Server::new()
    ///         .catches(group![not_found, internal_server])
    ///         .serve(3000)
    ///         .await
    /// }
    /// ```
    pub fn catches<const SIZE: usize>(mut self, catches: [Arc<dyn Catch>; SIZE]) -> Self {
        for catch in catches {
            self.router.catch(catch);
        }
        self
    }

    /// Serve the current router at the given socket
    ///
    /// This method returns a Future and should have `.await` called
    /// on it in an async method.
    ///
    /// # Example
    /// ```
    /// use wayfinder::server;
    ///
    /// #[wayfinder::main]
    /// async main() {
    ///     Server::new()
    ///         serve(3000)
    ///         .await
    /// }
    /// ```
    pub async fn serve<ADDR: IntoSocketAddr>(
        &mut self,
        addr: ADDR,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let addr: SocketAddr = addr.into_socket_addr();

        let listener = TcpListener::bind(addr.clone()).await?;
        println!("Server started at https://{}", addr);

        self.router.serve_routes();

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            let rh = self.router.clone();

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(|req| rh.parse(req)))
                    .await
                {
                    println!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}
