mod handler;

use std::{
    collections::HashMap, convert::Infallible, fmt::Debug, future::Future, pin::Pin, sync::Arc,
};

use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    service::Service,
    StatusCode,
};
use tokio::sync::Mutex;

use crate::{error::Error, response::IntoResponse};

use handler::{Handler, HandlerFuture};

#[derive(Default, Clone)]
pub enum Endpoint {
    #[default]
    None,
    Handler(Arc<dyn Handler<Future = HandlerFuture> + Send + Sync + 'static>),
}

impl Debug for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "No",
                Self::Handler(_) => "Yes",
            }
        )
    }
}

#[derive(Debug)]
pub struct Route {
    callbacks: RouteMethods,
}

#[derive(Debug)]
pub struct Routes(HashMap<String, Route>);

impl Routes {
    pub fn insert(&mut self, key: String, value: Route) -> Option<Route> {
        self.0.insert(key, value)
    }

    pub fn fetch(&self, uri: &str, method: &hyper::Method) -> Endpoint {
        if !self.0.contains_key(uri) {
            return Endpoint::None;
        }

        self.0.get(uri).unwrap().fetch(&method).clone()
    }

    pub fn new() -> Self {
        Routes(HashMap::new())
    }
}

#[doc = "Create a new route with the fallback handler"]
pub fn fallback<T>(callback: T) -> Route
where
    T: Handler<Future = HandlerFuture> + Send + Sync,
{
    Route {
        callbacks: RouteMethods {
            fallback: Endpoint::Handler(callback.referenced()),
            ..Default::default()
        },
    }
}

macro_rules! make_methods {
    ($($method: ident),*) => {
        paste::paste! {
            $(
                #[doc="Create a new route with the " $method " method handler"]
                pub fn [<$method:lower>]<T>(callback: T) -> $crate::server::router::Route
                where
                    T: Handler<Future = HandlerFuture> + Send + Sync,
                {
                    $crate::server::router::Route {
                        callbacks: $crate::server::router::RouteMethods {
                            [<$method:lower>]: $crate::server::router::Endpoint::Handler(callback.referenced()),
                            ..Default::default()
                        },
                    }
                }
            )*
        }
        paste::paste! {
            impl Route {
                pub fn fetch(&self, method: &hyper::Method) -> Endpoint {
                    use hyper::Method;
                    match method {
                        $(&Method::$method => match &self.callbacks.[<$method:lower>]{
                            // If endpoint doesn't exist use fallback
                            Endpoint::None => self.callbacks.fallback.clone(),
                            valid => valid.clone()
                        },)*
                        _ => Endpoint::None,
                    }
                }

                #[doc="Fallback method handler"]
                pub fn fallback<T>(mut self, callback: T) -> Self
                where
                    T: Handler<Future = HandlerFuture> + Send + Sync
                {
                    self.callbacks.fallback =
                        $crate::server::router::Endpoint::Handler(callback.referenced());
                    self
                }

                $(
                    #[doc=$method " method handler"]
                    pub fn [<$method:lower>]<T>(mut self, callback: T) -> Self
                    where
                        T: Handler<Future = HandlerFuture> + Send + Sync
                    {
                        self.callbacks.[<$method:lower>] =
                            $crate::server::router::Endpoint::Handler(callback.referenced());
                        self
                    }
                )*
            }
        }
        paste::paste! {
            #[derive(Default, Debug)]
            pub struct RouteMethods {
                $([<$method:lower>]: Endpoint,)*
                fallback: Endpoint,
            }
        }
    };
}

make_methods! {GET, POST, DELETE, PUT, HEAD, CONNECT, OPTIONS, TRACE, PATCH}

pub struct Builder {
    routes: Routes,
}

impl Builder {
    pub fn route(mut self, path: &str, route: Route) -> Self {
        self.routes.insert(path.to_string(), route);
        self
    }

    pub fn fallback<F>(self, fallback: F) -> Router
    where
        F: Handler<Future = HandlerFuture> + Send + Sync,
    {
        Router {
            handler: Endpoint::None,
            routes: Arc::new(Mutex::new(self.routes)),
            fallback: Endpoint::Handler(fallback.referenced()),
        }
    }

    pub fn build(self) -> Router {
        Router {
            handler: Endpoint::None,
            routes: Arc::new(Mutex::new(self.routes)),
            fallback: Endpoint::None,
        }
    }
}

/// Wrapper around important pointers to routing data.
///
/// Contained pointers:
/// - `Option<Handler>`: If present, this callback is the only handler that is used for all requests.
/// - `Routes`: The paths and their endpoints + fallback
/// - `Fallbacks`: The StatusCodes and their respective handlers.
#[derive(Clone)]
pub struct Router {
    pub handler: Endpoint,
    pub routes: Arc<Mutex<Routes>>,
    pub fallback: Endpoint,
}

impl Router {
    pub async fn handler(
        request: hyper::Request<Incoming>,
        routes: Arc<Mutex<Routes>>,
        handler: Endpoint,
        global_fallback: Endpoint,
    ) -> Result<hyper::Response<Full<Bytes>>, Infallible> {
        if let Endpoint::Handler(handler) = handler {
            return Ok(handler.call(request).await);
        }

        // Scoped fetch so lock is released after endpoint is fetched
        let endpoint = {
            let routes = routes.lock().await;
            routes.fetch(request.uri().path(), &request.method())
        };

        // TODO: add static file serving
        let result = match endpoint {
            Endpoint::Handler(endpoint) => endpoint.call(request).await,
            Endpoint::None => {
                if let Endpoint::Handler(fallback) = global_fallback {
                    fallback.call(request).await
                } else {
                    Error::from((StatusCode::NOT_FOUND, "Page not found")).into_response()
                }
            }
        };

        Ok(result)
    }

    pub fn new() -> Builder {
        Builder {
            routes: Routes::new(),
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
            self.handler.clone(),
            self.fallback.clone(),
        ))
    }
}

/// Convert object into a `tela::server::Router` object
pub trait IntoRouter {
    fn into_router(self) -> Router;
}

impl<F> IntoRouter for F
where
    F: Handler<Future = HandlerFuture> + Send + Sync,
{
    fn into_router(self) -> Router {
        Router {
            handler: Endpoint::Handler(self.referenced()),
            routes: Arc::new(Mutex::new(Routes::new())),
            fallback: Endpoint::None,
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
