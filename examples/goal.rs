extern crate web;
use bytes::Bytes;
use web::{prelude::*, Response, Router, Server};

#[tokio::main]
async fn main() {
    Server::new(([127, 0, 0, 1], 3000))
        .router(router! {
            "/" => message,
            "/hello" => hello,
        })
        .serve()
        .await;
}

struct Context;

trait Responder {
    fn into_response(self) -> bytes::Bytes;
}

type Result<T: Responder> = std::result::Result<T, u16>;

impl Responder for &str {
    fn into_response(self) -> bytes::Bytes {
        Bytes::from(self.to_string())
    }
}

#[route("/", methods=[get, post])]
fn message(_cx: Context) -> Result<&'static str> {
    // PERF: Support for return type of Responder.
    // templating with HandleBars and Tera
    // Macro based `rsx` / templating
    Ok(r#"<html>
        <head>
            <title>Home</title>
        </head>
        <body>
            <h1>Hello World</h1>
            <ul>
                <li>Welcome</li>
                <li>to</li>
                <li>LaunchPad</li>
            </ul>
        </body>
    </html>"#)
}

#[get("/")]
fn hello(_cx: Option<String>) -> Result<&'static str> {
    // "Hello".into()
    Err(500)
}

#[component]
fn button(_cx: Option<String>, name: &str) -> String {
    format!("<button>{}</button>", name)
}
