use std::{path::PathBuf, sync::Arc};

use tela::{
    server::{State, methods::*, Router, Server, Socket},
    sync::Mutex,
    Html,
};

use tera::Tera;

#[derive(Clone)]
struct AppState {
    tera: Arc<Mutex<Tera>>,
}

impl AppState {
    fn new(templates: &str) -> Self {
        let tera = match Tera::new(
            PathBuf::from(templates)
                .join("**/*.html")
                .display()
                .to_string()
                .as_str(),
        ) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Parsing error(s): {}", e);
                std::process::exit(1);
            }
        };

        AppState {
            tera: Arc::new(Mutex::new(tera)),
        }
    }
}

macro_rules! ctx {
    ($($name: ident: $value: literal),* $(,)?) => {
        {
            let mut _c = tera::Context::new();
            $(_c.insert(stringify!($name), $value);)*
            _c
        }
    };
}

#[tela::main]
async fn main() {
    let state = AppState::new("templates");

    Server::builder()
        .on_bind(|addr| println!("Serving at {}", addr))
        .serve(
            Socket::default(),
            Router::builder()
                .route(
                    "/",
                    get(|State(app_state): State<AppState>| async move {
                        let context = ctx! {
                            title: "Tera Example",
                            message: "Hello, Tera!",
                        };

                        let tera = app_state.tera.lock().await;
                        Html(tera.render("index.html", &context).unwrap())
                    }),
                )
                .state(state),
        )
        .await;
}
