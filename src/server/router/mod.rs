pub mod handler;
pub mod route;

pub use route::*;

use std::{
    collections::HashMap, convert::Infallible, fmt::Debug, future::Future, path::PathBuf, pin::Pin,
    sync::Arc,
};

use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    service::Service,
    StatusCode,
};
use tokio::sync::Mutex;

use crate::{error::Error, response::IntoResponse};

use handler::Handler;

use self::route::{BoxedHandler, Endpoint, IntoStaticPath, Route, Routes};

/// Router builder struct.
pub struct Builder {
    routes: Routes,
    assets: HashMap<String, PathBuf>,
}

impl Builder {
    /// Define a route to be handled by the router.
    ///
    /// The route handler is a collection of methods that have any number of arguments,
    /// 0 to 15, with each argument implementing `FromRequest` except the last parameter
    /// which must implement `FromRequestBody` which consumes the request.
    ///
    /// The route handler can be built from the helper methods that can be chained to add handlers
    /// for specific method types. Use the `any` method to create a handler for any method type.
    /// This can also be used as a sort of `"fallback"` for if a handler isn't found for the given
    /// method, a.k.a `404`.
    ///
    /// # Example
    /// ```
    /// use tela::server::{Router, router::get};
    ///
    /// async fn get_handler() {}
    /// async fn post_handler() {}
    /// async fn any_handler() {}
    ///
    /// async fn main() {
    ///     let _ = Router::new()
    ///         .route("/", get(get_handler).post(post_handler).any(any_handler))
    /// }
    /// ```
    pub fn route(mut self, path: &str, handler: Route) -> Self {
        self.routes.insert(path.to_string(), handler);
        self
    }

    /// Define a uri route to an asset path relationship.
    ///
    /// This route will be checked before handlers with the same route. Only if the file isn't found will the router then check the
    /// handlers.
    ///
    /// If a plain `&str` or `String` is passed in then the path and uri route are the same.
    /// Otherwise a tuple of `(route, path)` can be passed in.
    ///
    /// # Example
    /// ```
    /// #[tela::main]
    /// async fn main() {
    ///     let _ = Router::new()
    ///         .assets(("/images", "examples/assets"));
    /// }
    /// ```
    ///
    /// If a request was made for `/images/sample.png` then the `examples/assets/sample.png` file
    /// will be served.
    pub fn assets<S: IntoStaticPath>(mut self, path: S) -> Self {
        let (key, value) = path.into_static_path();
        if !value.exists() {
            eprintln!("The static asset path \"{}\" does not exist. It will be ignored until the path is generated.", value.display())
        }
        self.assets.insert(key, value);
        self
    }

    /// Assign a catch all handler for any request method.
    ///
    /// This handler is only called if a handler for any other defined route is not found. The
    /// handler passed in can be thought of as the last resort handler for a `404`.
    pub fn any<H, F>(self, handler: H) -> Router
    where
        H: Handler<F>,
        F: Send + Sync + 'static,
    {
        Router {
            handler: None,
            assets: self.assets.into(),
            routes: Arc::new(Mutex::new(self.routes)),
            any: Some(Endpoint::new(BoxedHandler::from_handler(handler))),
        }
    }

    pub fn build(self) -> Router {
        Router {
            handler: None,
            assets: self.assets.into(),
            routes: Arc::new(Mutex::new(self.routes)),
            any: None,
        }
    }
}

/// The heart of tela; the router takes each request and attempts to resolve the request to the
/// appropriate handler or file.
#[derive(Clone)]
pub struct Router {
    pub handler: Option<Endpoint>,
    pub assets: Arc<HashMap<String, PathBuf>>,
    pub routes: Arc<Mutex<Routes>>,
    pub any: Option<Endpoint>,
}

impl Router {
    pub async fn handler(
        request: hyper::Request<Incoming>,
        routes: Arc<Mutex<Routes>>,
        assets: Arc<HashMap<String, PathBuf>>,
        handler: Option<Endpoint>,
        any: Option<Endpoint>,
    ) -> Result<hyper::Response<Full<Bytes>>, Infallible> {
        if let Some(Endpoint(handler)) = handler {
            return Ok(handler.call(request, Captures::new()).await);
        }

        // Check for matching static asset path
        {
            let path = request.uri().path();
            for (key, value) in assets.iter() {
                if path.starts_with(key) {
                    let file_path = value.join(path.replace(key, ""));
                    if file_path.exists() {
                        let mime = mime_guess::from_path(&file_path).first_or_text_plain();
                        if let Ok(contents) = tokio::fs::read(&file_path).await {
                            let body = contents.into();
                            return Ok(hyper::Response::builder()
                                .status(200)
                                .header("Content-Type", mime.to_string())
                                .body(Full::new(body))
                                .unwrap());
                        }
                    }
                }
            }
        }

        // Scoped fetch so lock is released after endpoint is fetched
        let (endpoint, catches) = {
            let mut routes = routes.lock().await;
            match routes.fetch(request.uri().path(), &request.method()) {
                Some((callback, catches)) => (callback.clone(), catches.clone()),
                None => {
                    if let Some(endpoint) = &any {
                        (endpoint.clone(), Captures::new())
                    } else {
                        return Ok(
                            Error::from((StatusCode::NOT_FOUND, "Page not found")).into_response()
                        );
                    }
                }
            }
        };

        Ok(endpoint.0.call(request, catches).await)
    }

    /// Create a new router which can have builder methods chained.
    ///
    /// Use `route`, `assets`, and `any` to define different handlers for the router.
    pub fn builder() -> Builder {
        Builder {
            routes: Routes::new(),
            assets: HashMap::new(),
        }
    }
}

impl Debug for Router {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Router({:?})", self.routes)
    }
}

// Allow Router itself to handle hyper requests
impl Service<hyper::Request<Incoming>> for Router {
    type Response = hyper::Response<Full<Bytes>>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: hyper::Request<Incoming>) -> Self::Future {
        Box::pin(Router::handler(
            req,
            self.routes.clone(),
            self.assets.clone(),
            self.handler.clone(),
            self.any.clone(),
        ))
    }
}

/// Convert object into a `tela::server::Router` object
pub trait IntoRouter {
    fn into_router(self) -> Router;
}

impl<H> IntoRouter for H
where
    H: Handler + Send + Sync,
{
    fn into_router(self) -> Router {
        Router {
            handler: Some(Endpoint(Arc::new(BoxedHandler::from_handler(self)))),
            assets: Arc::new(HashMap::new()),
            routes: Arc::new(Mutex::new(Routes::new())),
            any: None,
        }
    }
}

impl IntoRouter for Router {
    fn into_router(self) -> Router {
        self
    }
}

impl IntoRouter for Builder {
    fn into_router(self) -> Router {
        self.build()
    }
}
