use std::{collections::HashMap, convert::Infallible, ffi::OsStr, fs, path::Path, sync::Arc};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Uri};
use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};

use crate::{
    request::{Catch, Endpoint},
    uri::index,
};

mod error {
    use phf::phf_map;

    /// Default http error messages
    pub static MESSAGES: phf::Map<u16, &'static str> = phf_map! {
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
}

/// Commands sent through channel to router
#[derive(Debug)]
pub enum Command {
    Get {
        method: Method,
        path: String,
        response: oneshot::Sender<Option<Route>>,
    },
    Error {
        code: u16,
        response: oneshot::Sender<Option<ErrorHandler>>,
    },
}

#[derive(Debug, Clone)]
pub struct Route(pub Arc<dyn Endpoint>);

#[derive(Debug, Clone)]
pub struct ErrorHandler(pub Arc<dyn Catch>);

#[derive(Clone)]
pub struct Router {
    channel: Option<Sender<Command>>,
    router: HashMap<Method, Vec<Route>>,
    catch: HashMap<u16, ErrorHandler>,
    assets: String,
}
impl Router {
    pub fn new() -> Self {
        Router {
            channel: None,
            router: HashMap::new(),
            catch: HashMap::new(),
            assets: "web/".to_string(),
        }
    }

    pub fn assets(&mut self, path: String) {
        self.assets = path;
    }

    pub fn catch(&mut self, catch: Arc<dyn Catch>) {
        if !self.catch.contains_key(&catch.code()) {
            self.catch.insert(catch.code(), ErrorHandler(catch));
        }
    }

    pub fn route(&mut self, route: Arc<dyn Endpoint>) {
        for method in route.methods() {
            if !self.router.contains_key(&method) {
                self.router.insert(method.clone(), Vec::new());
            }
            self.router
                .get_mut(&method)
                .unwrap()
                .push(Route(route.clone()));
        }
    }

    /// Start listener thread for handling access to router
    ///
    /// Creates mpsc channel and returns Sender handle. The thread that this method
    /// creates is the only instance of the router that should exists.
    pub fn serve_routes(&mut self) {
        let (tx, mut rx) = mpsc::channel::<Command>(32);
        let router = self.router.clone();
        let catch = self.catch.clone();

        tokio::spawn(async move {
            'watcher: while let Some(cmd) = rx.recv().await {
                use Command::*;

                match cmd {
                    Get {
                        method,
                        path,
                        response,
                    } => {
                        match router.get(&method) {
                            Some(data) => {
                                match index(
                                    &path,
                                    &data.iter().map(|r| r.0.path()).collect::<Vec<String>>(),
                                ) {
                                    Some(index) => {
                                        response.send(Some(data[index].clone())).unwrap();
                                        continue 'watcher;
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        };
                        response.send(None).unwrap();
                    }
                    Error { code, response } => {
                        if catch.contains_key(&code) {
                            response
                                .send(catch.get(&code).map(|eh| eh.clone()))
                                .unwrap()
                        } else if catch.contains_key(&0) {
                            response.send(catch.get(&0).map(|eh| eh.clone())).unwrap()
                        } else {
                            response.send(None).unwrap()
                        }
                    }
                }
            }
        });

        self.channel = Some(tx);
    }

    async fn error(
        &self,
        uri: &Uri,
        method: &Method,
        code: u16,
        reason: String,
        channel: Sender<Command>,
    ) -> std::result::Result<hyper::Response<Full<Bytes>>, Infallible> {
        let (error_tx, error_rx) = oneshot::channel();
        match channel
            .send(Command::Error {
                code: code.clone(),
                response: error_tx,
            })
            .await
        {
            Ok(_) => {}
            Err(error) => eprintln!("{:?}", error),
        };

        match error_rx.await.unwrap() {
            Some(ErrorHandler(handler)) => {
                match handler.execute(
                    code.clone(),
                    error::MESSAGES.get(&code).unwrap_or(&"").to_string(),
                    reason.clone(),
                ) {
                    Ok(response) => {
                        Router::log_request(
                            &uri.path().to_string(),
                            &method.clone(),
                            &response.status().into(),
                        );
                        Ok(response)
                    }
                    Err((code, reason)) => {
                        Router::log_request(&uri.path().to_string(), method, &code);
                        Ok(hyper::Response::builder()
                            .status(code)
                            .header("Wayfinder-Reason", reason)
                            .body(Full::new(Bytes::new()))
                            .unwrap())
                    }
                }
            }
            None => {
                Router::log_request(&uri.path().to_string(), method, &code);
                Ok(hyper::Response::builder()
                    .status(code)
                    .header("Wayfinder-Reason", reason)
                    .body(Full::new(Bytes::new()))
                    .unwrap())
            }
        }
    }

    fn log_request(path: &String, method: &Method, status: &u16) {
        #[cfg(debug_assertions)]
        eprintln!(
            "  {}(\x1b[3{}m{}\x1b[39m) \x1b[32m{:?}\x1b[0m",
            method,
            match status {
                100..=199 => 6,
                200..=299 => 2,
                300..=399 => 5,
                400..=499 => 1,
                500..=599 => 3,
                _ => 7,
            },
            status,
            path
        )
    }

    pub async fn parse(
        &self,
        request: hyper::Request<hyper::body::Incoming>,
    ) -> Result<hyper::Response<Full<Bytes>>, Infallible> {
        // Get all needed information from request
        let mut uri = request.uri().clone();
        let method = request.method().clone();
        // Can be used for validation, authentication, and other features
        let _headers = request.headers().clone();
        let mut body = request.collect().await.unwrap().to_bytes().to_vec();

        let (endpoint_tx, endpoint_rx) = oneshot::channel();
        match &self.channel {
            Some(channel) => {
                let path = format!("{}{}", self.assets, uri.path());
                let path = Path::new(&path);
                if let Some(extension) = path.extension().and_then(OsStr::to_str) {
                    match fs::read_to_string(path) {
                        Ok(text) => {
                            Router::log_request(&uri.path().to_string(), &method, &200);
                            let mut builder = hyper::Response::builder().status(200);

                            match mime_guess::from_ext(extension).first() {
                                Some(mime) => {
                                    builder = builder.header("Content-Type", mime.to_string())
                                }
                                _ => {}
                            };

                            return Ok(builder.body(Full::new(Bytes::from(text))).unwrap());
                        }
                        _ => {
                            Router::log_request(&uri.path().to_string(), &method, &404);
                            return Ok(hyper::Response::builder()
                                .status(404)
                                .header("Wayfinder-Reason", "File not found")
                                .body(Full::new(Bytes::new()))
                                .unwrap());
                        }
                    }
                }

                match channel
                    .send(Command::Get {
                        method: method.clone(),
                        path: uri.path().to_string(),
                        response: endpoint_tx,
                    })
                    .await
                {
                    Ok(_) => {}
                    Err(error) => eprintln!("{}", error),
                };

                match endpoint_rx.await.unwrap() {
                    Some(Route(endpoint)) => match endpoint.execute(&mut uri, &mut body) {
                        Ok(response) => {
                            Router::log_request(
                                &uri.path().to_string(),
                                &method,
                                &response.status().into(),
                            );
                            Ok(response)
                        }
                        Err((code, reason)) => {
                            self.error(&uri, &method, code, reason, channel.clone())
                                .await
                        }
                    },
                    None => {
                        self.error(
                            &uri,
                            &method,
                            404,
                            "Page not found in router".to_string(),
                            channel.clone(),
                        )
                        .await
                    }
                }
            }
            _ => panic!("Unable to communicate with router"),
        }
    }
}
