extern crate new;

use html_to_string_macro::html;
use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    Request, Response,
};
use new::server::{error::Error, router::Router, serve, Socket};

fn handler(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, Error> {
    Ok(Response::new(Full::new(Bytes::from("Hello, world!"))))
}

fn hours(_: Request<Incoming>) -> String {
    html! {
        <html>
            <head>
                <script>"
                    async function getPost() {
                        console.log(await (await fetch('/hours',{method: 'POST'})).text());
                    }
                "</script>
                <title>"Example"</title>
            </head>
            <body>
                <h1>"GET"</h1>
                <h2>"Hours Page"</h2>
                <button onclick="getPost()">"POST"</button>
            </body>
        </html>
    }
}

macro_rules! router {
    ($(
        $path: literal : {
            $init: ident : $init_handler: expr
        }
     ),* $(,)?) => {
        {
            Router::new()
                $(
                    .route($path, new::server::router::$init($init_handler)$(.$method($handler))*)
                )*
        }
    };
    ($(
        $path: literal : {
            $init: ident : $init_handler: expr,
            $($method: ident : $handler: expr),* $(,)?
        }),* $(,)?) => {
        {
            Router::new()
                $(
                    .route($path, new::server::router::$init($init_handler)$(.$method($handler))*)
                )*
        }
    };
}

#[tokio::main]
async fn main() {
    let _ = serve(
        Socket::Local(3000),
        router! {
            "/hours": {
                post: |_| "(POST) Hours Page!!!",
                get: hours
            }
        },
    )
    .await;
}
