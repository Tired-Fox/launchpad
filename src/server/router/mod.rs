pub mod handler;

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

/// Wrapper around a route handler pointer.
#[derive(Clone)]
struct BoxedHandler<I>(Arc<dyn Handler<I>>);

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
        (self.0).handle(request).await
    }
}

/// Allows the dynamic route handler pointer to be called.
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

/// Wrapper around a route handler.
#[derive(Clone)]
pub struct Endpoint(Arc<dyn ErasedHandler>);

impl Debug for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Endpoint",)
    }
}

/// A wrapper that holds handlers for a given route.
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

/// A wrapper arround a mapping routes to their handlers.
#[derive(Debug)]
pub struct Routes(HashMap<String, Route>);

impl Routes {
    pub fn insert(&mut self, key: String, value: Route) -> Option<Route> {
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

#[doc = "Create a new route with the handler that handles any request method"]
pub fn any<H, T>(handler: H) -> Route
where
    H: Handler<T>,
    T: Send + Sync + 'static,
{
    Route {
        callbacks: RouteMethods {
            any: Some(Endpoint(Arc::new(BoxedHandler::from_handler(handler)))),
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
                /// Merge duplicate route paths together. New handlers override old handlers.
                fn merge(&mut self, new: Route) {
                    $(Route::replace_or_not(&mut self.callbacks.[<$method:lower>], new.callbacks.[<$method:lower>]);)*
                }

                pub fn fetch(&self, method: &hyper::Method) -> Option<&Endpoint> {
                    use hyper::Method;
                    match method {
                        $(&Method::$method => match &self.callbacks.[<$method:lower>]{
                            // If endpoint doesn't exist use fallback
                            None => self.callbacks.any.as_ref(),
                            Some(valid) => Some(valid)
                        },)*
                        _ => None,
                    }
                }

                #[doc="Any method handler"]
                pub fn any<H, T>(mut self, handler: H) -> Self
                where
                    H: Handler<T>,
                    T: Send + Sync + 'static
                {
                    self.callbacks.any =
                        Some($crate::server::router::Endpoint(Arc::new(BoxedHandler::from_handler(handler))));
                    self
                }

                $(
                    #[doc=$method " method handler"]
                    pub fn [<$method:lower>]<H, T>(mut self, handler: H) -> Self
                    where
                        H: Handler<T>,
                        T: Send + Sync + 'static
                    {
                        self.callbacks.[<$method:lower>] =
                            Some($crate::server::router::Endpoint(Arc::new(BoxedHandler::from_handler(handler))));
                        self
                    }
                )*
            }
        }
        paste::paste! {
            /// All method handlers for a given route.
            #[derive(Default, Debug)]
            pub struct RouteMethods {
                $([<$method:lower>]: Option<Endpoint>,)*
                any: Option<Endpoint>,
            }
        }
    };
}

make_methods! {GET, POST, DELETE, PUT, HEAD, CONNECT, OPTIONS, TRACE, PATCH}

/// Convert a path into a uri path starting with `/`.
///
/// # Example
/// ```
/// "some/path\\here" -> "/some/path/here"
/// ```
pub(crate) fn to_uri(uri: &String) -> String {
    let mut uri = uri.replace("\\", "/").replace("//", "/");
    if !uri.starts_with("/") {
        uri = String::from("/") + uri.as_str();
    }
    uri
}

pub trait IntoStaticPath {
    fn into_static_path(self) -> (String, PathBuf);
}

impl IntoStaticPath for String {
    fn into_static_path(self) -> (String, PathBuf) {
        let uri = to_uri(&self);
        (uri, PathBuf::from(self))
    }
}

impl IntoStaticPath for &str {
    fn into_static_path(self) -> (String, PathBuf) {
        let uri = to_uri(&self.to_string());
        (uri, PathBuf::from(self))
    }
}

impl<S1: ToString, S2: ToString> IntoStaticPath for (S1, S2) {
    fn into_static_path(self) -> (String, PathBuf) {
        let uri = to_uri(&self.0.to_string());
        (uri, PathBuf::from(self.1.to_string()))
    }
}

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
            any: Some(Endpoint(Arc::new(BoxedHandler::from_handler(handler)))),
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
            return Ok(handler.call(request).await);
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
        let endpoint = {
            let routes = routes.lock().await;
            match routes.fetch(request.uri().path(), &request.method()) {
                Some(callback) => callback.clone(),
                None => {
                    if let Some(endpoint) = &any {
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
