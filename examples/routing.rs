extern crate web;

use web::{prelude::*, Server, Router, Response};

#[tokio::main]
async fn main() {
    Server::new(([127, 0, 0, 1], 3000))
        .router(routes! {
            ["/": get, post] => message,
            ["/hello": get] => hello,
        })
        .serve()
        .await;
}

// #[route("/", methods=[get, post])]
fn message(_cx: Option<String>) -> Response {
    // PERF: Support for return type of Responder.
    // templating with HandleBars and Tera
    // Macro based `rsx` / templating
    r#"<html>
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
    </html>"#.into()
}

fn hello(_cx: Option<String>) -> Response {
    // "Hello".into()
    500.into()
}
