use http_body_util::Full;
pub use hyper::Method;
use hyper::{server::conn::http1, service::service_fn};
use std::{convert::Infallible, net::SocketAddr, path::PathBuf, fs};

use bytes::Bytes;
use std::sync::{Arc, Mutex};
use tokio::{
    net::TcpListener,
    sync::{
        mpsc::{self, Sender},
        oneshot,
    },
};

use super::router::Router;
use crate::{Response, RouteCallback};

pub struct Server {
    addr: SocketAddr,
    router: Arc<Mutex<Router>>,
}

macro_rules! response {
    ($data: expr) => {
       hyper::Response::new(http_body_util::Full::new(bytes::Bytes::from($data)))
    };
}

macro_rules! error {
    ($code: expr, $msg: expr) => {
       hyper::Response::builder()
           .status($code)
           .body(http_body_util::Full::new(bytes::Bytes::from($msg)))
           .unwrap() 
    };
    ($code: expr) => {
       hyper::Response::builder()
           .status($code)
           .body(http_body_util::Full::new(bytes::Bytes::new()))
           .unwrap() 
    };
}

#[derive(Debug)]
enum Command {
    Get {
        method: Method,
        path: String,
        response: oneshot::Sender<Option<RouteCallback>>,
    },
    Error {
        code: u16,
        response: oneshot::Sender<String>,
    },
}

impl Server {
    pub fn new(addr: impl Into<SocketAddr>) -> Self {
        Server {
            addr: addr.into(),
            router: Arc::new(Mutex::new(Router::new())),
        }
    }

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
                    Error { code, response } => {
                        let router = router.lock().unwrap();
                        response.send(router.get_error(code)).unwrap()
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

    pub fn router(self, router: Router) -> Self {
        Server {
            router: Arc::new(Mutex::new(router)),
            ..self
        }
    }
}

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
                    path,
                    response: resp_tx,
                })
                .await
                .unwrap();

            let endpoint = resp_rx.await.unwrap();
            match endpoint {
                // PERF: Pass in data to callback \/
                Some(callback) => match callback(None) {
                    Response::Success(data) => hyper::Response::new(Full::new(data)),
                    Response::Error(code) => {
                        let (resp_tx, resp_rx) = oneshot::channel();
                        router
                            .send(Command::Error {
                                code: code.clone(),
                                response: resp_tx,
                            })
                            .await
                            .unwrap();
                        let message = resp_rx.await.unwrap();
                        error!(code, message)
                    }
                },
                _ => {
                    // PERF: Should remove this. Not sure if this will be wanted.
                    if path_buff.with_extension("js").is_file() {
                        response!(fs::read_to_string(path_buff.to_str().unwrap()).expect("Could not read from file"))
                    } else {
                        let (resp_tx, resp_rx) = oneshot::channel();
                        router
                            .send(Command::Error {
                                code: 404,
                                response: resp_tx,
                            })
                            .await
                            .unwrap();
                        let message = resp_rx.await.unwrap();
                        response!(message)
                    }
                }
            }
        },
        Some(_) => {
            if !path_buff.is_file() {
                error!(404)
            } else {
                response!(fs::read_to_string(path).expect("Could not read from file"))
            }
        }
    };

    Ok(response)
}
