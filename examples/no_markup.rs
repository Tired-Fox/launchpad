use tela::{
    html::{self},
    server::{
        router::{get, Captures},
        Router, Server, Socket,
    },
    Html,
};

async fn handler(catches: Captures) -> Html<String> {
    println!("{:?}", catches);
    html::into!(<h1>"Hello, world!"</h1>)
}

#[tela::main]
async fn main() {
    Server::builder()
        .on_bind(|addr| println!("Serving at {}", addr))
        .serve(
            Socket::Local(3000),
            Router::builder().route("/blog/:...subpage/defs/:page", get(handler)),
        )
        .await;
}
