use std::fmt::{Display, Formatter};
use std::ops::DerefMut;
use std::sync::Arc;

use tela::html;
use tela::server::{methods::get, Router, Server, Socket, State};
use tela::sync::Mutex;

struct Count(pub i64);
impl Count {
    fn inc(&mut self) {
        if self.0 < i64::MAX - 1 {
            self.0 += 1;
        } else {
            self.0 = 0;
        }
    }
}

impl Display for Count {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
struct AppState {
    name: &'static str,
    count: Arc<Mutex<Count>>,
}

#[tela::main]
async fn main() {
    // Anything that will be mutated should be wrapped in Arc and Mutex for Clone/Use across threads and thread safety respectively.
    let state = AppState {
        name: "Tela",
        count: Arc::new(Mutex::new(Count(0))),
    };

    Server::builder()
        .on_bind(|addr| println!("Serving to {}", addr))
        .serve(
            Socket::Local(3000),
            Router::builder().state(state).route(
                "/",
                get(|State(app_state): State<AppState>| async move {
                    let mut count = app_state.count.lock().await;
                    count.deref_mut().inc();
                    html::new!(<h1>{format!("[{}] Hello, {}!", count, app_state.name)}</h1>)
                }),
            ),
        )
        .await;
}

