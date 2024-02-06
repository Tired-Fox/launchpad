use tela::websocket::{connect, upgrade, Websocket, Message};
use std::{convert::Infallible, future::Future, net::SocketAddr, pin::Pin};

use anyhow::Result;
use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    server::conn::http1,
    Method, Request, Response,
};
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use tokio::net::TcpListener;

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

/// Handle a websocket connection.
async fn serve_websocket(websocket: Websocket) -> Result<()> {
    let mut websocket = websocket.await?;
    while let Some(message) = websocket.next().await {
        match message? {
            Message::Text(msg) => {
                println!("Received text message: {msg}");
                websocket.send(Message::text("Thank you, come again.")).await?;
            }
            Message::Binary(msg) => {
                println!("Received binary message: {msg:02X?}");
                websocket.send(Message::binary(b"Thank you, come again.".to_vec())).await?;
            }
            Message::Ping(msg) => {
                // No need to send a reply: tungstenite takes care of this for you.
                println!("Received ping message: {msg:02X?}");
            }
            Message::Pong(msg) => {
                println!("Received pong message: {msg:02X?}");
            }
            Message::Close(msg) => {
                // No need to send a reply: tungstenite takes care of this for you.
                if let Some(msg) = &msg {
                    println!("Received close message with code {} and message: {}", msg.code, msg.reason);
                } else {
                    println!("Received close message");
                }
            }
            Message::Frame(_msg) => {
                unreachable!();
            }
        }
    }

    Ok(())
}

#[derive(Clone)]
struct Router;

impl Router {
    async fn handler(
        mut req: Request<Incoming>,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        match upgrade(&mut req, None) {
            // Websocket: upgrade connection
            Ok((res, ws)) => {
                tokio::spawn(async move {
                    // Use websocket handler
                    if let Err(err) = serve_websocket(ws).await {
                        println!("Error in websocket: {err:?}");
                    }
                });
                Ok(res)
            }
            // normal connection
            Err(_) => {
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
    }
}

impl tower::Service<Request<Incoming>> for Router {
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
        Box::pin(Router::handler(req))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    #[cfg(debug_assertions)]
    println!("Serving at {}", addr);

    tokio::task::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();

            let io = TokioIo::new(stream);
            let service = TowerToHyperService::new(Router);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new().keep_alive(true).serve_connection(io, service).with_upgrades().await {
                    eprintln!("Error serving connection: {:?}", err)
                }
            });
        }
    });

    let mut ws = connect("ws://127.0.0.1:3000", None::<u8>, None).await?.await?;
    ws.send(Message::text("Hello")).await?;
    for i in 0..5 {
        if let Some(message) = ws.next().await {
            match message? {
                Message::Text(data) => {
                    println!("Received message: {data}");
                    ws.send(Message::text(i.to_string())).await?;
                }
                _ => {}
            }
        }
    }
    ws.send(Message::Close(None)).await?;
    Ok(())
}
