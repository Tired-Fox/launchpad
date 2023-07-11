extern crate launchpad;

use launchpad::{prelude::*, Server};

#[tokio::main]
async fn main() {
    Server::new(([127, 0, 0, 1], 3000))
        .router(routes! {
            "/" => message,
            "/hello" => hello
        })
        .serve()
        .await;
}

#[get]
fn message() -> Result<&'static str> {
    // PERF: Support for return type of Responder.
    // templating with HandleBars and Tera
    // Macro based `rsx` / templating
    Ok(r#"<html lang="en">
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

#[get]
fn hello() -> Result<&'static str> {
    // "Hello".into()
    Err(500)
}
