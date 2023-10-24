use tela::{
    html,
    request::State,
    server::{
        router::{get, Captures},
        Router, Server, Socket,
    },
    Html,
};

async fn handler(catches: Captures, State(app_state): State<AppState>) -> Html<String> {
    println!("{:?}", app_state.name);
    println!("{:?}", catches);
    html::into!(<h1>"Hello, world!"</h1>)
}

#[derive(Clone)]
struct AppState {
    pub name: &'static str,
}

#[tela::main]
async fn main() {
    let state = AppState { name: "Tela" };

    Server::builder()
        .on_bind(|addr| println!("Serving at {}", addr))
        .serve(
            Socket::Local(3000),
            Router::builder()
                .route("/blog/:...subpage/defs/:page", get(handler))
                .with_state(state),
        )
        .await;
}
