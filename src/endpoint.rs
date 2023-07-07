use http_body_util::Full;
pub use hyper::Method;
use hyper::{body::Incoming, server::conn::http1, service::service_fn, Response};
use std::{collections::HashMap, convert::Infallible, fmt::Display, net::SocketAddr};

use bytes::Bytes;
use std::sync::{Arc, Mutex};
use tokio::{
    net::TcpListener,
    sync::{
        mpsc::{self, Sender},
        oneshot,
    },
};

pub type RequestCallback = fn(hyper::Request<Incoming>) -> Result<Bytes, (u16, String)>;

pub struct Function(RequestCallback);
impl Function {
    pub fn new(callback: RequestCallback) -> Self {
        Function(callback)
    }
}

#[macro_export]
macro_rules! method {
    (get) => {
        Method::GET
    };
    (post) => {
        Method::POST
    };
    (delete) => {
        Method::DELETE
    };
    (put) => {
        Method::PUT
    };
    (head) => {
        Method::HEAD
    };
    (options) => {
        Method::OPTIONS
    };
    (connect) => {
        Method::CONNECT
    };
    (trace) => {
        Method::TRACE
    };
    (patch) => {
        Method::PATCH
    };
}

// pub fn create_router()

#[macro_export]
macro_rules! routes {
    { $([$path: literal$($methods:tt)*] => $callback: ident),* $(,)?} => {
        Router::from([
            $(
                (
                    routes!(@methods $($methods)*),
                    $path.to_string(),
                    $callback as $crate::endpoint::RequestCallback,
                )
            ),*
        ])
    };
    ( @methods $(:)+ $($method: ident),*) => {
        vec![$($crate::method!($method),)*]
    };
    ( @methods) => {};
}

pub struct Server {
    addr: SocketAddr,
    router: Arc<Mutex<Router>>,
}

#[derive(Debug)]
enum Command {
    Get {
        method: Method,
        path: String,
        response: oneshot::Sender<Option<RequestCallback>>,
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
                use Command::Get;

                match cmd {
                    Get {
                        method,
                        path,
                        response,
                    } => {
                        let router = router.lock().unwrap();
                        response
                            .send(router.get(method, path).map(|f| f.clone()))
                            .unwrap();
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
) -> Result<Response<Full<Bytes>>, Infallible> {
    let (resp_tx, resp_rx) = oneshot::channel();
    router
        .send(Command::Get {
            method: req.method().clone(),
            path: req.uri().path().to_string(),
            response: resp_tx,
        })
        .await
        .unwrap();

    let endpoint = resp_rx.await.unwrap();
    let response = match endpoint {
        Some(callback) => match callback(req) {
            Ok(response) => Response::new(Full::new(response)),
            Err((code, message)) => Response::builder()
                .status(code)
                .body(Full::new(Bytes::from(message)))
                .unwrap(),
        },
        // TODO: Add logic for getting user defined error handlers
        _ => Response::new(Full::new(Bytes::from("<h1>404 Not Found</h1>"))),
    };

    // let response = match path {
    //     "/" => Response::new(Full::new(Bytes::from("Hello, World!"))),
    //     "/hello" => {
    //         println!("Getting hello file");

    //         let contents = fs::read_to_string("web/hello.html").unwrap();
    //         Response::new(Full::new(Bytes::from(contents)))
    //     }
    //     _ => {
    //         let contents = fs::read_to_string("web/404.html").unwrap();
    //         Response::new(Full::new(Bytes::from(contents)))
    //     }
    // };
    Ok(response)
}

pub struct Request {
    methods: Vec<Method>,
    callback: RequestCallback,
}

impl Request {
    pub fn new(methods: Vec<Method>, callback: RequestCallback) -> Self {
        Request { methods, callback }
    }
}

/// Endpoint => handler relationship
/// where handler has certain request methods it can run with
#[derive(Debug, Clone)]
pub struct Router(HashMap<Method, HashMap<String, RequestCallback>>);

impl<const SIZE: usize> From<[(Vec<Method>, String, RequestCallback); SIZE]> for Router {
    fn from(value: [(Vec<Method>, String, RequestCallback); SIZE]) -> Self {
        let mut router = Router::new();
        for val in value {
            router.set(Request::new(val.0, val.2), val.1)
        }
        router
    }
}

impl Router {
    fn new() -> Self {
        Router(HashMap::new())
    }

    fn get<S: Display>(&self, method: Method, path: S) -> Option<&RequestCallback> {
        let path = path.to_string();
        match self.0.get(&method) {
            Some(bucket) => bucket.get(&path),
            _ => None,
        }
    }

    /// Map an endpoint given the request type.
    ///
    /// If the mapping already exists it will be overridden
    fn set<S>(&mut self, req: Request, path: S)
    where
        S: Display,
    {
        for method in req.methods {
            match self.0.get_mut(&method) {
                Some(bucket) => {
                    bucket.insert(path.to_string(), req.callback);
                }
                None => {
                    self.0.insert(method.clone(), HashMap::new());
                    self.0
                        .get_mut(&method)
                        .unwrap()
                        .insert(path.to_string(), req.callback);
                }
            }
        }
    }
}
