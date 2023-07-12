use http_body_util::Full;
pub use hyper::Method;
use hyper::{server::conn::http1, service::service_fn};
use std::{convert::Infallible, fs, net::SocketAddr, path::PathBuf};

use bytes::Bytes;
use std::sync::{Arc, Mutex};
use tokio::{
    net::TcpListener,
    sync::{
        mpsc::{self, Sender},
        oneshot,
    },
};

use super::{
    router::{Route, Router},
    Response,
};

/// Commands sent through channel to router
#[derive(Debug)]
enum Command {
    Get {
        method: Method,
        path: String,
        response: oneshot::Sender<Option<Route>>,
    },
    Error {
        code: u16,
        reason: String,
        response: oneshot::Sender<String>,
    },
}

fn format_body(body: String) -> String {
    let re = regex::Regex::new("(?i)<meta +charset=\"UTF-8\" *>").unwrap();
    let response = match re.find(body.as_str()) {
        None => format!("<head><meta charset=\"UTF-8\"></head>\n{}", body),
        Some(_) => body,
    };
    response
}

fn format_request_debug(
    message: String,
    request: &hyper::Request<hyper::body::Incoming>,
) -> String {
    format!(
        r#"<pre>
{}
    <em>Request: <strong>{} → '{}</strong>'</em>
    <em>Body: <strong>{:?}</strong></em>
</pre>"#,
        message,
        request.method(),
        request.uri(),
        request.body()
    )
}

macro_rules! response {
    ($data: expr) => {
        hyper::Response::new(http_body_util::Full::new(bytes::Bytes::from(format_body(
            $data,
        ))))
    };
}

fn build_error(code: u16, msg: String, reason: String) -> hyper::Response<Full<Bytes>> {
    let mut response = hyper::Response::builder().status(code);
    match code {
        301 | 308 | 302 | 303 | 307 => {
            if reason == String::new() {
                panic!("Redirect responses require an error reason that is the Location")
            }
            response = response
                .header("LaunchPad-Reason", "Redirect")
                .header("Location", reason);
        }
        _ => {
            response = response.header("LaunchPad-Reason", reason);
        }
    }

    response
        .body(http_body_util::Full::new(bytes::Bytes::from(
            match msg.len() == 0 {
                false => format_body(msg),
                _ => msg,
            },
        )))
        .unwrap()
}
macro_rules! error {
    ($code: expr, $reason: expr, $msg: expr) => {
        build_error($code, $msg, $reason)
    };
    ($code: expr, $reason: expr) => {
        build_error($code, String::new(), $reason)
    };
    ($code: expr) => {
        build_error($code, String::new(), String::new())
    };
}

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
    addr: SocketAddr,
    router: Arc<Mutex<Router>>,
}

impl Server {
    /// Create a new server with a given address
    ///
    /// The method can take anything that can be converted into a SocketAddr
    ///
    /// # Example
    /// ```rust
    /// use launchpad::{prelude::*, Server};
    ///
    /// fn main() {
    ///     Server::new(([127, 0, 0, 1], 3000))
    ///         .serve()
    ///         .await;
    /// }
    /// ```
    ///
    /// ```rust
    /// use launchpad::{prelude::*, Server};
    ///
    /// fn main() {
    ///     Server::new("127.0.0.1:3000")
    ///         .serve()
    ///         .await;
    /// }
    /// ```
    pub fn new(addr: impl Into<SocketAddr>) -> Self {
        Server {
            addr: addr.into(),
            router: Arc::new(Mutex::new(Router::new())),
        }
    }

    /// Starts the server and handles requests
    ///
    /// # Example
    /// ```rust
    /// use launchpad::{prelude::*, Server};
    ///
    /// fn main() {
    ///     Server::new("127.0.0.1:3000")
    ///         .serve()
    ///         .await;
    /// }
    /// ```
    pub async fn serve(&self) {
        let listener = TcpListener::bind(self.addr).await.unwrap();
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

        println!("Listening on http://{}", self.addr);
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let router = tx.clone();
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(stream, service_fn(|req| handler(req, router.clone())))
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

/// Core request handler
async fn handler(
    req: hyper::Request<hyper::body::Incoming>,
    router: Sender<Command>,
) -> Result<hyper::Response<Full<Bytes>>, Infallible> {
    let mut path = req.uri().path().to_string();
    if path.ends_with("/") {
        path.pop();
    }
    let path_buff = PathBuf::from(path.clone());

    let response = match path_buff.extension() {
        None => {
            let (resp_tx, resp_rx) = oneshot::channel();
            router
                .send(Command::Get {
                    method: req.method().clone(),
                    path: path.clone(),
                    response: resp_tx,
                })
                .await
                .unwrap();

            let endpoint = resp_rx.await.unwrap();
            match endpoint {
                Some(endpoint) => match endpoint.endpoint().call(&req) {
                    Response::Success(data) => hyper::Response::new(Full::new(data)),
                    Response::Error(code, message) => {
                        let (resp_tx, resp_rx) = oneshot::channel();
                        router
                            .send(Command::Error {
                                code: code.clone(),
                                reason: format_request_debug(match &message {
                                    Some(msg) => msg.clone(),
                                    _ => String::from("A user defined error occured"),
                                }, &req),
                                response: resp_tx,
                            })
                            .await
                            .unwrap();

                        let body = resp_rx.await.unwrap();
                        match message {
                            Some(msg) => error!(code, msg, body),
                            _ => error!(code, "".to_string(), body),
                        }
                    }
                },
                _ => {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    router
                        .send(Command::Error {
                            code: 404,
                            reason: format_request_debug(
                                format!(
                                    "<span class=\"path\"><strong>/{}</strong></span> not found in router",
                                    path_buff.to_string_lossy()
                                ),
                                &req
                            ),
                            response: resp_tx,
                        })
                        .await
                        .unwrap();
                    let message = resp_rx.await.unwrap();
                    error!(404, "path not found in router".to_string(), message)
                }
            }
        }
        Some(_) => {
            if !path_buff.is_file() {
                if path_buff.to_str().unwrap().ends_with("html") {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    router
                        .send(Command::Error {
                            code: 404,
                            reason: format_request_debug(
                                format!("Could not find file {:?}", path_buff.to_string_lossy()),
                                &req,
                            ),
                            response: resp_tx,
                        })
                        .await
                        .unwrap();
                    let message = resp_rx.await.unwrap();
                    error!(
                        404,
                        format!("File not found: {}", path_buff.to_string_lossy()),
                        message
                    )
                } else {
                    error!(404, "Path not in router".to_string())
                }
            } else {
                response!(fs::read_to_string(path).expect("Could not read from file"))
            }
        }
    };

    Ok(response)
}
