use std::{path::PathBuf, sync::Arc};

use tela::{
    server::{State, methods::*, Router, Server, Socket},
    sync::Mutex,
    Html,
};

use handlebars::Handlebars;

#[derive(Clone)]
struct AppState {
    hbrs: Arc<Mutex<Handlebars<'static>>>,
}

impl AppState {
    fn new(templates: &str) -> Self {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_templates_directory(".hbs", PathBuf::from(templates))
            .unwrap();

        AppState {
            hbrs: Arc::new(Mutex::new(handlebars)),
        }
    }
}

macro_rules! ctx {
    ($($name: ident: $value: literal),* $(,)?) => {
        std::collections::BTreeMap::from([
            $((stringify!($name), $value),)*
        ])
    };
}

#[tela::main]
async fn main() {
    let state = AppState::new("templates/");

    Server::builder()
        .on_bind(|addr| println!("Serving at {}", addr))
        .serve(
            Socket::Local(3000),
            Router::builder()
                .route(
                    "/",
                    get(|State(app_state): State<AppState>| async move {
                        let context = ctx! {
                            title: "Tera Example",
                            message: "Hello, Handlebars!",
                        };

                        let hbrs = app_state.hbrs.lock().await;
                        Html(hbrs.render("index", &context).unwrap())
                    }),
                )
                .state(state),
        )
        .await;
}
