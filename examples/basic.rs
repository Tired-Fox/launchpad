use std::{convert::Infallible, future::Future, net::SocketAddr, pin::Pin};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::task::{Context, Poll};

use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    Method,
    Request, Response, server::conn::http1,
};
use hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN;
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use tokio::net::TcpListener;
use tower::{Layer, Service, ServiceBuilder, ServiceExt};
use tower::layer::util::Stack;
use tower_http::cors::{Any, CorsLayer};

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

#[derive(Debug, Clone)]
struct PathRouter;

#[derive(Debug, Clone)]
struct AssetRouter;

#[derive(Debug, Clone)]
struct Router {
    routes: PathRouter,
    assets: AssetRouter,
}

impl Router {
    fn new() -> Self {
        Self {
            routes: PathRouter,
            assets: AssetRouter,
        }
    }
}

impl Router {
    async fn handler(
        mut req: Request<Incoming>,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        #[cfg(debug_assertions)]
        println!("{} {} {:?}", req.method(), req.uri().path(), req.version());
        Ok(match (req.uri().path(), req.method()) {
            ("/", &Method::GET) => Response::new(Full::new(Bytes::from("Home Page"))),
            ("/hello", &Method::GET) => Response::new(Full::new(Bytes::from("Hello, world!"))),
            ("/data", &Method::POST) => {
                Response::builder()
                    .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                    .body(Full::new(Bytes::from(r#"{"name":"tela"}"#)))
                    .unwrap()
            }
            _ => Response::builder()
                .status(404)
                .body(Full::new(Bytes::from(ERROR_PAGE)))
                .unwrap(),
        })
    }
}

impl Service<Request<Incoming>> for Router {
    type Response = Response<Full<Bytes>>;
    type Error = Infallible;
    type Future =
    Pin<Box<dyn Future<Output=std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        Box::pin(async move {
            Middleware::new()
                .cors(
                    Cors::new()
                        .allow_origin(Any)
                        .allow_headers(Any)
                        .allow_methods(Any)
                )
                .service_fn(Router::handler)
                .ready()
                .await
                .unwrap().call(req).await
        })
    }
}

#[cfg(feature = "middleware")]
trait CorsLayerBuilder<L> {
    fn cors(self, cors: CorsLayer) -> ServiceBuilder<Stack<CorsLayer, L>>;
}

#[cfg(feature = "middleware")]
#[cfg_attr(docsrs, doc(cfg(feature = "middleware")))]
impl<L> CorsLayerBuilder<L> for ServiceBuilder<L> {
    fn cors(self, cors: CorsLayer) -> ServiceBuilder<Stack<CorsLayer, L>> {
        self.layer(cors)
    }
}

type Cors = CorsLayer;
type Middleware<L> = ServiceBuilder<L>;

struct Layered<L, H, T> {
    layer: L,
    handler: H,
    _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<L, H, T> Debug for Layered<L, H, T>
    where
        L: Debug, {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layered")
            .field("layer", &self.layer)
            .finish()
    }
}

impl<L, H, T> Clone for Layered<L, H, T>
    where
        L: Clone,
        H: Clone,
{
    fn clone(&self) -> Self {
        Self {
            layer: self.layer.clone(),
            handler: self.handler.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

trait IntoResponse {}

impl<L, H, T> Handler<T> for Layered<L, H, T>
    where
        L: Layer<HandlerService<H, T>> + Clone + Send + 'static,
        H: Handler<T>,
        L::Service: Service<Request<Incoming>, Error=Infallible, Response=Response<Full<Bytes>>> + Clone + Send + 'static,
        <L::Service as Service<Request<Incoming>>>::Future: Send,
        T: 'static,
{
    fn resolve(self, req: Request<Incoming>) -> impl Future<Output=std::result::Result<Response<Full<Bytes>>, Infallible>> + Send {
        let mut svc = self.layer.layer(HandlerService::new(self.handler));
        svc.call(req)
    }
}

trait Handler<T>: Clone + Send + Sized + 'static {
    fn resolve(self, req: Request<Incoming>) -> impl Future<Output=std::result::Result<Response<Full<Bytes>>, Infallible>> + Send;
    fn layer<L>(self, layer: L) -> Layered<L, Self, T>
        where
            L: Layer<HandlerService<T, Self>> + Clone,
            L::Service: Service<Request<Incoming>>
    {
        Layered {
            layer,
            handler: self,
            _phantom: std::marker::PhantomData,
        }
    }
}

struct HandlerService<H, T> {
    handler: H,
    _marker: PhantomData<fn() -> T>,
}

impl<H, T> HandlerService<H, T> {
    fn new(handler: H) -> Self {
        Self {
            handler,
            _marker: PhantomData,
        }
    }
}

impl<T, H: Handler<T>> Service<Request<Incoming>> for HandlerService<H, T> {
    type Response = Response<Full<Bytes>>;
    type Error = Infallible;
    type Future =
    Pin<Box<dyn Future<Output=std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        let handler = self.handler.clone();
        Box::pin(async move {
            handler.resolve(req).await
        })
    }
}

async fn serve<A>(addr: A, router: Router) -> Result<()>
    where
        SocketAddr: From<A>
{
    let addr = SocketAddr::from(addr);
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Serving at {}", addr);
    let service = TowerToHyperService::new(router.clone());

    loop {
        let (stream, _) = listener.accept().await.unwrap();

        let io = TokioIo::new(stream);
        let handler = service.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, handler).await {
                eprintln!("Error serving connection: {:?}", err)
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    serve(([127, 0, 0, 1], 3000), Router::new()).await
}
