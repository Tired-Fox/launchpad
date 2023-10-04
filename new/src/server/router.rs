use std::{
    collections::HashMap,
    fmt::Debug,
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
};

use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    service::Service,
    Request, Response,
};

use crate::response::IntoResponse;

use super::error::Error;

pub trait Handler: Send {
    fn call(&self, request: Request<Incoming>) -> Response<Full<Bytes>>;
    fn arced(self) -> Arc<dyn Handler + Send + Sync>;
}

impl<F, Res> Handler for F
where
    F: Fn(Request<Incoming>) -> Res + Sync + Send + 'static,
    Res: IntoResponse,
{
    fn call(&self, request: Request<Incoming>) -> Response<Full<Bytes>> {
        self(request).into_response()
    }

    fn arced(self) -> Arc<dyn Handler + Send + Sync> {
        Arc::new(self)
    }
}

#[derive(Default, Clone)]
pub enum Endpoint {
    #[default]
    None,
    Route(Arc<dyn Handler + Send + Sync + 'static>),
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
                pub fn [<$method:lower>]<T: Handler + Send + Sync>(callback: T) -> $crate::server::router::Route
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
                    pub fn [<$method:lower>]<T: Handler + Send + Sync>(mut self, callback: T) -> Self {
                        self.callbacks.[<$method:lower>] =
                            $crate::server::router::Endpoint::Route(callback.arced());
                        self
                    }
                )*
            }
        }
        paste::paste! {
            #[derive(Default, Clone)]
            pub struct RouteMethods {
                $([<$method:lower>]: Endpoint,)*
            }

            impl Debug for RouteMethods {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let se = |e: &Endpoint| match e {
                        &Endpoint::None => "None",
                        &Endpoint::Route(_) => "Route",
                    };
                    write!(
                        f,
                        "Methods({})",
                        vec![$((stringify!([<$method:lower>]:), se(&self.[<$method:lower>])),)*]
                            .iter()
                            .map(|kv: &(&str, &str)| {format!("{} {}", kv.0, kv.1)})
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                }
            }
        }
    };
}

make_methods! {GET, POST, DELETE, PUT, HEAD, CONNECT, OPTIONS, TRACE, PATCH}

#[derive(Clone)]
pub struct Router {
    pub handler: Option<
        Arc<dyn Fn(Request<Incoming>) -> Result<Response<Full<Bytes>>, Error> + Send + Sync>,
    >,
    pub routes: Arc<RwLock<Routes>>,
}

impl Router {
    pub async fn handler(
        handler: Option<
            Arc<dyn Fn(Request<Incoming>) -> Result<Response<Full<Bytes>>, Error> + Send + Sync>,
        >,
        request: Request<Incoming>,
        routes: Arc<RwLock<Routes>>,
    ) -> Result<Response<Full<Bytes>>, Error> {
        if let Some(handler) = handler {
            return handler(request);
        }

        let routes = routes.read().unwrap();
        match routes.fetch(&request.uri().to_string(), &request.method()) {
            // TODO: add static file serving
            Endpoint::None => Ok(Response::builder()
                .status(404)
                .header("Tela-Reason", "Page not found")
                .body(Full::new(Bytes::new()))
                .unwrap()),
            Endpoint::Route(endpoint) => Ok(endpoint.call(request).into_response()),
        }
    }

    pub fn new() -> Self {
        Router {
            handler: None,
            routes: Arc::new(RwLock::new(Routes::new())),
        }
    }

    pub fn spawn(&self) -> Self {
        Router {
            handler: self.handler.clone(),
            routes: self.routes.clone(),
        }
    }

    pub fn route(self, path: &str, route: Route) -> Self {
        {
            let mut routes = self.routes.write().unwrap();
            routes.insert(path.to_string(), route);
        }
        self
    }
}

impl Debug for Router {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Router({:?})", self.routes)
    }
}

// Allow Router itself to handle hyper requests
impl Service<Request<Incoming>> for Router {
    type Response = Response<Full<Bytes>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
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
    F: Fn(Request<Incoming>) -> Result<Response<Full<Bytes>>, Error> + Send + Sync + 'static,
{
    fn into_router(self) -> Router {
        Router {
            handler: Some(Arc::new(self)),
            routes: Arc::new(RwLock::new(Routes::new())),
        }
    }
}

impl IntoRouter for Router {
    fn into_router(self) -> Router {
        self
    }
}
