use tela::{
    server::{methods::*, Router, Server, Socket},
    Html,
};

use askama::*;

#[derive(Template)]
#[template(path = "index.html")]
struct HelloAskama<'a> {
    title: &'a str,
    message: &'a str,
}

#[tela::main]
async fn main() {
    Server::builder()
        .on_bind(|addr| println!("Serving at {}", addr))
        .serve(
            Socket::default(),
            Router::builder().route(
                "/",
                get(|| async move {
                    let template = HelloAskama {
                        title: "Askama Example",
                        message: "Hello, Askama!",
                    };
                    Html(template.render().unwrap())
                }),
            ),
        )
        .await;
}
