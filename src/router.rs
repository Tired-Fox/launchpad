use phf::phf_map;
use std::{collections::HashMap, fmt::Display, sync::Arc};

use hyper::Method;

use super::endpoint::Endpoint;

/// Default http error messages
static ERROR_MESSAGES: phf::Map<u16, &'static str> = phf_map! {
    100u16 => "Continue",
    101u16 => "Switching protocols",
    102u16 => "Processing",
    103u16 => "Early Hints",

    200u16 => "OK",
    201u16 => "Created",
    202u16 => "Accepted",
    203u16 => "Non-Authoritative Information",
    204u16 => "No Content",
    205u16 => "Reset Content",
    206u16 => "Partial Content",
    207u16 => "Multi-Status",
    208u16 => "Already Reported",
    226u16 => "IM Used",

    300u16 => "Multiple Choices",
    301u16 => "Moved Permanently",
    302u16 => "Found (Previously \"Moved Temporarily\")",
    303u16 => "See Other",
    304u16 => "Not Modified",
    305u16 => "Use Proxy",
    306u16 => "Switch Proxy",
    307u16 => "Temporary Redirect",
    308u16 => "Permanent Redirect",

    400u16 => "Bad Request",
    401u16 => "Unauthorized",
    402u16 => "Payment Required",
    403u16 => "Forbidden",
    404u16 => "Not Found",
    405u16 => "Method Not Allowed",
    406u16 => "Not Acceptable",
    407u16 => "Proxy Authentication Required",
    408u16 => "Request Timeout",
    409u16 => "Conflict",
    410u16 => "Gone",
    411u16 => "Length Required",
    412u16 => "Precondition Failed",
    413u16 => "Payload Too Large",
    414u16 => "URI Too Long",
    415u16 => "Unsupported Media Type",
    416u16 => "Range Not Satisfiable",
    417u16 => "Expectation Failed",
    418u16 => "I'm a Teapot",
    421u16 => "Misdirected Request",
    422u16 => "Unprocessable Entity",
    423u16 => "Locked",
    424u16 => "Failed Dependency",
    425u16 => "Too Early",
    426u16 => "Upgrade Required",
    428u16 => "Precondition Required",
    429u16 => "Too Many Requests",
    431u16 => "Request Header Fields Too Large",
    451u16 => "Unavailable For Legal Reasons",

    500u16 => "Internal Server Error",
    501u16 => "Not Implemented",
    502u16 => "Bad Gateway",
    503u16 => "Service Unavailable",
    504u16 => "Gateway Timeout",
    505u16 => "HTTP Version Not Supported",
    506u16 => "Variant Also Negotiates",
    507u16 => "Insufficient Storage",
    508u16 => "Loop Detected",
    510u16 => "Not Extended",
    511u16 => "Network Authentication Required",
};

/// Construct a router given a list of routes
///
/// # Example
///
/// Assume that the following method is in both examples
/// ```
/// #[get("/")]
/// fn home() -> Result<&'static str> {
///     Ok("Hello, world!")
/// }
/// ```
///
/// `routes!` can be used like the `vec!` macro
/// ```
/// use launchpad::prelude::*;
///
/// let router = routes![home]
/// ```
///
/// If you want to specify the `route/uri` for the endpoint in the macro you can
/// use it similar to a map macro.
/// ```
/// use launchpad::prelude::*;
///
/// let router = routes!{
///     "/": home
/// }
/// ```
#[macro_export]
macro_rules! routes {
    { $($path: literal => $endpoint: ident),* $(,)?} => {
        $crate::Router::from([
            $(
                $crate::router::Route::new(
                    $path.to_string(),
                    std::sync::Arc::new($endpoint(std::sync::Mutex::new($crate::state::State::default())))
                ),
            )*
        ])
    };
    [ $($endpoint: ident),* $(,)?] => {
        $crate::Router::from([
            $(
                $crate::router::Route::from_endpoint(
                    std::sync::Arc::new(
                        $endpoint( std::sync::Mutex::new($crate::state::State::default()) )
                    )
                ),
            )*
        ])
    }
}

/// A constructed and initialized route that is linked to an endpoint
#[derive(Debug)]
pub struct Route(String, Arc<dyn Endpoint>);

impl Route {
    /// Create a new route give a path and an endpoint
    pub fn new(path: String, endpoint: Arc<dyn Endpoint>) -> Self {
        Route(path, endpoint)
    }

    /// Create a new route with only an endpoint
    pub fn from_endpoint(value: Arc<dyn Endpoint>) -> Self {
        Route::new(value.path().clone(), value)
    }

    pub fn endpoint(&self) -> &Arc<dyn Endpoint> {
        &self.1
    }

    pub fn endpoint_mut(&mut self) -> &mut Arc<dyn Endpoint> {
        &mut self.1
    }

    pub fn path(&self) -> &String {
        &self.0
    }
}

impl Clone for Route {
    fn clone(&self) -> Self {
        Route(self.0.clone(), self.1.clone())
    }

    fn clone_from(&mut self, source: &Self) {
        self.0 = source.0.clone();
        self.1 = source.1.clone();
    }
}

/// A mapping of uri to endpoints and errors to error handlers
///
/// Currently it is mapped in this way that the endpoint is shared across
/// the different request methods. Soon it will be updated to filter by method
/// then by best match path. This will cause more lookup time and cost but should
/// future proof the router to handle complex features like getting props/params from
/// the uri and parsing Forms, etc...
///
/// ```plaintext
/// GET:
///     "/" -> home
/// POST:
///     "/" -> home
/// ```
///
/// to
///
/// ```plaintext
/// "/" -> home
/// ```
#[derive(Debug, Clone)]
pub struct Router {
    routes: HashMap<Method, Vec<Route>>,
    errors: HashMap<u16, fn() -> String>,
}

// <HEAP> [hello("/api/name/<first>/<last>"), world("/api/<...path>/help")] <- endpoints
//
// <routes: HashMap>
//  hyper::Method::GET <- [*hello, *world]
//  hyper::Method::POST <- [*world]
//
// GET "/api/name/<first>/<last>"
//  - routes.get(hyper::Method::GET) <- [*hello, *world]
//  - [*hello, *world].iter() <- Compare uri for closest match first
//      - Exact same Length
//      - Matching literals
//      - Ranked from best match to worst match

impl<const SIZE: usize> From<[Route; SIZE]> for Router {
    fn from(value: [Route; SIZE]) -> Self {
        let mut router = Router::new();
        for val in value {
            router.set_route(val.path().clone(), val)
        }
        router
    }
}

impl Router {
    /// Create a new blank router
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// Get an endpoint that best matches the request
    pub fn get_route<S: Display>(&self, method: Method, path: S) -> Option<&Route> {
        // TODO: use new uri matching
        let path = path.to_string();
        match self.routes.get(&method) {
            Some(bucket) => {
                let result = launchpad_uri::find(&path, &bucket, |s| s.path().clone());
                result
            },
            _ => None,
        }
    }

    /// Get an error message
    pub fn get_error(&self, code: u16) -> String {
        match self.errors.get(&code) {
            Some(callback) => callback(),
            _ => match ERROR_MESSAGES.get(&code) {
                Some(message) => {
                    format!(
                        r#"
<h1 style="text-align: center">{} {}</h1>
<div style="border-top: 1px solid black; margin-inline: 2rem"></div>"#,
                        code, message
                    )
                }
                _ => String::new(),
            },
        }
    }

    /// Set an error handler
    pub fn set_error(&mut self, code: u16, callback: fn() -> String) {
        self.errors.insert(code, callback);
    }

    /// Map an endpoint given the request type.
    ///
    /// If the mapping already exists it will be overridden
    pub fn set_route<S: Display>(&mut self, path: S, mut req: Route) {
        let mut path = path.to_string();
        if path.ends_with("/") {
            path.pop();
        }
        req.0 = path;

        for method in req.endpoint().methods() {
            match self.routes.get_mut(&method) {
                Some(bucket) => {
                    bucket.push(req.clone());
                }
                None => {
                    self.routes.insert(method.clone(), Vec::new());
                    self.routes
                        .get_mut(&method)
                        .unwrap()
                        .push(req.clone());
                }
            }
        }
    }
}
