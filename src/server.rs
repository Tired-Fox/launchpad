use http_body_util::{BodyExt, Full};
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

use crate::ROOT;

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
    method: &hyper::Method,
    uri: &hyper::Uri,
    body: &Bytes,
) -> String {
    format!(
        r#"<pre>
{}
    <em>Request: <strong>{} → '{}</strong>'</em>
    <em>Body: <strong>{}</strong></em>
</pre>"#,
        message,
        method,
        uri,
        String::from_utf8(body.to_vec()).unwrap()
    )
}

macro_rules! response {
    ($data: expr) => {
        hyper::Response::new(http_body_util::Full::new(bytes::Bytes::from(format_body(
            $data,
        ))))
    };
    (FILE $data: expr) => {
        hyper::Response::new(http_body_util::Full::new(bytes::Bytes::from($data)))
    };
}

fn build_error(code: u16, msg: String, reason: String, fb: bool) -> hyper::Response<Full<Bytes>> {
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
                false => match fb {
                    true => format_body(msg),
                    _ => msg,
                },
                _ => msg,
            },
        )))
        .unwrap()
}
macro_rules! error {
    ($code: expr, $reason: expr, $msg: expr) => {
        build_error($code, $msg, $reason, true)
    };
    ($code: expr, $reason: expr) => {
        build_error($code, String::new(), $reason, true)
    };
    ($code: expr) => {
        build_error($code, String::new(), String::new(), true)
    };
    (RAW $code: expr, $reason: expr, $msg: expr) => {
        build_error($code, $msg, $reason, false)
    };
    (RAW $code: expr, $reason: expr) => {
        build_error($code, String::new(), $reason, false)
    };
    (RAW $code: expr) => {
        build_error($code, String::new(), String::new(), false)
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

        let message = "http://";
        let fill = (0..self.addr.to_string().len() + message.len() + 16)
            .map(|_| '╌')
            .collect::<String>();
        println!(
            "{}",
            format!(
                "
╭{}╮
╎ 🚀 \x1b[33;1mLaunchpad\x1b[39;22m: {}{} ╎
╰{}╯
",
                fill, message, self.addr, fill
            )
        );
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

    let uri = req.uri().clone();
    let headers = req.headers().clone();
    let method = req.method().clone();
    let body = req.collect().await.unwrap().to_bytes();

    let response = match path_buff.extension() {
        None => {
            let (resp_tx, resp_rx) = oneshot::channel();
            router
                .send(Command::Get {
                    method: method.clone(),
                    path: path.clone(),
                    response: resp_tx,
                })
                .await
                .unwrap();

            let endpoint = resp_rx.await.unwrap();
            match endpoint {
                Some(endpoint) => match endpoint.endpoint().execute(&uri, &headers, &body) {
                    Response::Success(content_type, data) => hyper::Response::builder()
                        .status(200)
                        .header("Content-Type", content_type)
                        .body(Full::new(data))
                        .unwrap(),
                    Response::Error(code, message) => {
                        let (resp_tx, resp_rx) = oneshot::channel();
                        router
                            .send(Command::Error {
                                code: code.clone(),
                                reason: format_request_debug(
                                    match &message {
                                        Some(msg) => msg.clone(),
                                        _ => String::from("A user defined error occured"),
                                    },
                                    &method,
                                    &uri,
                                    &body,
                                ),
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
                    println!("{}", path_buff.display());
                    router
                        .send(Command::Error {
                            code: 404,
                            reason: format_request_debug(
                                format!(
                                    "<span class=\"path\"><strong>{}{}</strong></span> not found in router",
                                    match path_buff.display().to_string().as_str() {
                                        "" => "/",
                                        _ => ""
                                    },
                                    path_buff.display(),
                                ),
                                &method,
                                &uri,
                                &body
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
            let path_buff = PathBuf::from(format!("{}{}", ROOT, path));

            if !path_buff.is_file() {
                if path_buff.to_str().unwrap().ends_with("html") {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    router
                        .send(Command::Error {
                            code: 404,
                            reason: format_request_debug(
                                format!("Could not find file {:?}", path_buff.to_string_lossy()),
                                &method,
                                &uri,
                                &body,
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
                    error!(RAW 404, "Path not in router".to_string())
                }
            } else {
                response!(
                    FILE
                    fs::read(path_buff).expect("Could not read from file")
                )
            }
        }
    };

    #[cfg(debug_assertions)]
    {
        let code = response.status().as_u16();
        println!(
            "  {} ({}) {}",
            match method {
                Method::GET => format!("\x1b[36mGET\x1b[39m"),
                Method::POST => format!("\x1b[35mPOST\x1b[39m"),
                Method::DELETE => format!("\x1b[31mDELETE\x1b[39m"),
                val => format!("{}", val),
            },
            match code {
                100..=199 => format!("\x1b[36m{}\x1b[39m", code),
                200..=299 => format!("\x1b[32m{}\x1b[39m", code),
                300..=399 => format!("\x1b[33m{}\x1b[39m", code),
                400..=499 => format!("\x1b[31m{}\x1b[39m", code),
                500..=599 => format!("\x1b[35m{}\x1b[39m", code),
                _ => code.to_string(),
            },
            format!("\x1b[32m'{}'\x1b[39m", path)
        );
    }
    Ok(response)
}
