mod handler;

use std::{collections::HashMap, fmt::Debug, future::Future, pin::Pin, sync::Arc};

use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    service::Service,
};
use tokio::sync::Mutex;

use crate::{
    response::{IntoResponse, Response},
    server::error::Error,
    Request,
};

pub use handler::Handler;
use handler::HandlerFuture;

#[derive(Default, Clone)]
pub enum Endpoint {
    #[default]
    None,
    Route(Arc<dyn Handler<Future = HandlerFuture> + Send + Sync + 'static>),
}

impl Debug for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "None",
                Self::Route(_) => "Route",
            }
        )
    }
}

#[derive(Clone, Debug)]
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

        let route = self.0.get(uri).unwrap();
        route.fetch(method)
    }

    pub fn new() -> Self {
        Routes(HashMap::new())
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
                            [<$method:lower>]: $crate::server::router::Endpoint::Route(callback.arced()),
                            ..Default::default()
                        }
                    }
                }
            )*
        }
        paste::paste! {
            impl Route {
                pub fn fetch(&self, method: &hyper::Method) -> Endpoint {
                    use hyper::Method;
                    match method {
                        $(&Method::$method => self.callbacks.[<$method:lower>].clone(),)*
                        _ => Endpoint::None,
                    }
                }

                $(
                    #[doc=$method " method handler"]
                    pub fn [<$method:lower>]<T>(mut self, callback: T) -> Self
                    where
                        T: Handler<Future = HandlerFuture> + Send + Sync
                    {
                        self.callbacks.[<$method:lower>] =
                            $crate::server::router::Endpoint::Route(callback.arced());
                        self
                    }
                )*
            }
        }
        paste::paste! {
            #[derive(Default, Clone, Debug)]
            pub struct RouteMethods {
                $([<$method:lower>]: Endpoint,)*
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

    pub fn build(self) -> Router {
        Router {
            handler: None,
            routes: Arc::new(Mutex::new(self.routes)),
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
    pub handler: Option<Arc<dyn Fn(Request) -> Response + Send + Sync>>,
    pub routes: Arc<Mutex<Routes>>,
}

impl Router {
    pub async fn handler(
        handler: Option<Arc<dyn Fn(Request) -> Response + Send + Sync>>,
        request: hyper::Request<Incoming>,
        routes: Arc<Mutex<Routes>>,
    ) -> Result<hyper::Response<Full<Bytes>>, Error> {
        if let Some(handler) = handler {
            return Ok(handler(request.into()).into_response());
        }

        let routes = routes.lock().await;
        match routes.fetch(&request.uri().to_string(), &request.method()) {
            // TODO: add static file serving
            Endpoint::None => Ok(hyper::Response::builder()
                .status(404)
                .header("Tela-Reason", "Page not found")
                .body(Full::new(Bytes::new()))
                .unwrap()),
            Endpoint::Route(endpoint) => {
                let endpoint = endpoint.clone();
                Ok(endpoint.call(request).await)
            }
        }
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
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: hyper::Request<Incoming>) -> Self::Future {
        Box::pin(Router::handler(
            self.handler.clone(),
            req,
            self.routes.clone(),
        ))
    }
}

/// Convert object into a `tela::server::Router` object
pub trait IntoRouter {
    fn into_router(self) -> Router;
}

impl<F> IntoRouter for F
where
    F: Fn(Request) -> Response + Send + Sync + 'static,
{
    fn into_router(self) -> Router {
        Router {
            handler: Some(Arc::new(self)),
            routes: Arc::new(Mutex::new(Routes::new())),
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
