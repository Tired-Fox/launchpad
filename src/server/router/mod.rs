pub mod handler;

use std::{
    collections::HashMap, convert::Infallible, fmt::Debug, future::Future, pin::Pin,
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

#[derive(Clone)]
pub struct BoxedHandler<I>(Arc<dyn Handler<I>>);

impl<I> BoxedHandler<I>
where
    I: Send + Sync + 'static,
{
    pub fn from_handler<H>(handler: H) -> Self
    where
        H: Handler<I>,
    {
        BoxedHandler(Arc::new(handler))
    }

    pub async fn call(&self, request: hyper::Request<Incoming>) -> hyper::Response<Full<Bytes>> {
        (self.0).handle_request(request).await
    }
}

pub trait ErasedHandler: Send + Sync + 'static {
    fn call(
        &self,
        request: hyper::Request<Incoming>,
    ) -> Pin<Box<dyn Future<Output = hyper::Response<Full<Bytes>>> + Send + '_>>;
}

impl<I> ErasedHandler for BoxedHandler<I>
where
    I: Send + Sync + 'static,
{
    fn call(
        &self,
        request: hyper::Request<Incoming>,
    ) -> Pin<Box<dyn Future<Output = hyper::Response<Full<Bytes>>> + Send + '_>> {
        Box::pin(self.call(request))
    }
}

#[derive(Clone)]
pub struct Endpoint(Arc<dyn ErasedHandler>);

impl Debug for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Endpoint",)
    }
}

#[derive(Debug)]
pub struct Route {
    callbacks: RouteMethods,
}

impl Route {
    fn replace_or_not(endpoint: &mut Option<Endpoint>, new: Option<Endpoint>) {
        if let Some(_) = &new {
            *endpoint = new
        }
    }
}

#[derive(Debug)]
pub struct Routes(HashMap<String, Route>);

impl Routes {
    pub fn insert(&mut self, key: String, value: Route) -> Option<Route> {
        // TODO: Auto merge duplicate routes. Routes from incoming value overrides duplicate
        // methods
        if self.0.contains_key(&key) {
            self.0.get_mut(&key).unwrap().merge(value);
            None
        } else {
            self.0.insert(key, value)
        }
    }

    pub fn fetch(&self, uri: &str, method: &hyper::Method) -> Option<&Endpoint> {
        if !self.0.contains_key(uri) {
            return None;
        }

        self.0.get(uri).unwrap().fetch(&method)
    }

    pub fn new() -> Self {
        Routes(HashMap::new())
    }
}

#[doc = "Create a new route with the fallback handler"]
pub fn fallback<H, T>(callback: H) -> Route
where
    H: Handler<T>,
    T: Send + Sync + 'static,
{
    Route {
        callbacks: RouteMethods {
            fallback: Some(Endpoint(Arc::new(BoxedHandler::from_handler(callback)))),
            ..Default::default()
        },
    }
}

macro_rules! make_methods {
    ($($method: ident),*) => {
        paste::paste! {
            $(
                #[doc="Create a new route with the " $method " method handler"]
                pub fn [<$method:lower>]<H, T>(callback: H) -> $crate::server::router::Route
                where
                    H: Handler<T>,
                    T: Send + Sync + 'static
                {
                    $crate::server::router::Route {
                        callbacks: $crate::server::router::RouteMethods {
                            [<$method:lower>]: Some($crate::server::router::Endpoint(Arc::new(BoxedHandler::from_handler(callback)))),
                            ..Default::default()
                        },
                    }
                }
            )*
        }
        paste::paste! {
            impl Route {
                fn merge(&mut self, new: Route) {
                    $(Route::replace_or_not(&mut self.callbacks.[<$method:lower>], new.callbacks.[<$method:lower>]);)*
                }

                pub fn fetch(&self, method: &hyper::Method) -> Option<&Endpoint> {
                    use hyper::Method;
                    match method {
                        $(&Method::$method => match &self.callbacks.[<$method:lower>]{
                            // If endpoint doesn't exist use fallback
                            None => self.callbacks.fallback.as_ref(),
                            Some(valid) => Some(valid)
                        },)*
                        _ => None,
                    }
                }

                #[doc="Fallback method handler"]
                pub fn fallback<H, T>(mut self, callback: H) -> Self
                where
                    H: Handler<T>,
                    T: Send + Sync + 'static
                {
                    self.callbacks.fallback =
                        Some($crate::server::router::Endpoint(Arc::new(BoxedHandler::from_handler(callback))));
                    self
                }

                $(
                    #[doc=$method " method handler"]
                    pub fn [<$method:lower>]<H, T>(mut self, callback: H) -> Self
                    where
                        H: Handler<T>,
                        T: Send + Sync + 'static
                    {
                        self.callbacks.[<$method:lower>] =
                            Some($crate::server::router::Endpoint(Arc::new(BoxedHandler::from_handler(callback))));
                        self
                    }
                )*
            }
        }
        paste::paste! {
            #[derive(Default, Debug)]
            pub struct RouteMethods {
                $([<$method:lower>]: Option<Endpoint>,)*
                fallback: Option<Endpoint>,
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

    pub fn fallback<H, F>(self, fallback: H) -> Router
    where
        H: Handler<F>,
        F: Send + Sync + 'static,
    {
        Router {
            handler: None,
            routes: Arc::new(Mutex::new(self.routes)),
            fallback: Some(Endpoint(Arc::new(BoxedHandler::from_handler(fallback)))),
        }
    }

    pub fn build(self) -> Router {
        Router {
            handler: None,
            routes: Arc::new(Mutex::new(self.routes)),
            fallback: None,
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
    pub handler: Option<Endpoint>,
    pub routes: Arc<Mutex<Routes>>,
    pub fallback: Option<Endpoint>,
}

impl Router {
    pub async fn handler(
        request: hyper::Request<Incoming>,
        routes: Arc<Mutex<Routes>>,
        handler: Option<Endpoint>,
        global_fallback: Option<Endpoint>,
    ) -> Result<hyper::Response<Full<Bytes>>, Infallible> {
        if let Some(Endpoint(handler)) = handler {
            return Ok(handler.call(request).await);
        }

        // Scoped fetch so lock is released after endpoint is fetched
        // TODO: add static file serving
        let endpoint = {
            let routes = routes.lock().await;
            match routes.fetch(request.uri().path(), &request.method()) {
                Some(callback) => callback.clone(),
                None => {
                    if let Some(endpoint) = &global_fallback {
                        endpoint.clone()
                    } else {
                        return Ok(
                            Error::from((StatusCode::NOT_FOUND, "Page not found")).into_response()
                        );
                    }
                }
            }
        };

        Ok(endpoint.0.call(request).await)
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

impl<H> IntoRouter for H
where
    H: Handler + Send + Sync,
{
    fn into_router(self) -> Router {
        Router {
            handler: Some(Endpoint(Arc::new(BoxedHandler::from_handler(self)))),
            routes: Arc::new(Mutex::new(Routes::new())),
            fallback: None,
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
