use std::{convert::Infallible, future::Future, net::SocketAddr, pin::Pin};

use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    server::conn::http1,
    Method, Request, Response,
};
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use tokio::net::TcpListener;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

static ERROR_PAGE: &str = r#"
<html>
    <head>
        <style>
            html {
                background-color: rgb(26 26 26);
                color: rgb(232 232 232);
                font-family: arial;
            }
            a {
                display: block;
                color: inherit;
                width: fit-content;
                margin-inline: auto;
            }
            h1 {
                text-align: center;
                margin-block: 2rem;
            }
        </style>
    </head>
    <body>
    <h1>404 Not Found</h1>
    <a href="/">Back to Home</a>
    </body>
</html>
"#;

#[derive(Clone)]
struct Router;
impl Router {
    async fn handler(
        mut req: Request<Incoming>,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        #[cfg(debug_assertions)]
        println!("{} {} {:?}", req.method(), req.uri().path(), req.version());
        Ok(match (req.uri().path(), req.method()) {
            ("/", &Method::GET) => Response::new(Full::new(Bytes::from("Home Page"))),
            ("/hello", &Method::GET) => Response::new(Full::new(Bytes::from("Hello, world!"))),
            _ => Response::builder()
                .status(404)
                .body(Full::new(Bytes::from(ERROR_PAGE)))
                .unwrap(),
        })
    }
}

impl tower::Service<Request<Incoming>> for Router {
    type Response = Response<Full<Bytes>>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        Box::pin(Router::handler(req))
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();

    #[cfg(debug_assertions)]
    println!("Serving at {}", addr);

    loop {
        let (stream, _) = listener.accept().await.unwrap();

        let io = TokioIo::new(stream);
        let service = TowerToHyperService::new(Router);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("Error serving connection: {:?}", err)
            }
        });
    }
}
