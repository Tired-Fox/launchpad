extern crate launchpad;
use std::sync::{Arc, Mutex, MutexGuard};

use launchpad::router;
use launchpad::v2::state::Empty;
use launchpad::{
    prelude::*,
    v2::{
        endpoint::{Context, Endpoint, Result},
        state::State,
        Response, Server,
    },
};

#[tokio::main]
async fn main() {
    Server::new(([127, 0, 0, 1], 3000))
        .router(router![world])
        .serve()
        .await;
}

#[derive(Debug, Default)]
struct WorldState {
    pub name: String,
    pub count: u16,
}

#[get("/hello-world")]
fn world(state: &mut State<WorldState>) -> Result<String> {
    // PERF: Get a better way of manipulating state so it doesn't block for too long
    state.inner_mut().count += 1;
    if state.inner().name == "".to_string() {
        state.inner_mut().name = "Zachary".to_string();
    }

    Ok(format!("Hello World, and {}: {} times", state.inner().name, state.inner().count))
}

#[post("/")]
fn home() -> Result<&'static str> {
    Ok("Home")
}

// #[derive(Debug)]
// struct World(Mutex<State<WorldState>>);
// impl Endpoint for World {
//     fn methods(&self) -> Vec<Method> {
//         vec![Method::GET]
//     }

//     fn path(&self) -> String {
//         String::from("/")
//     }

//     fn call(&self) -> Response {
//         fn endpoint_call(state: &mut State<WorldState>) -> Result<String> {
//             state.inner_mut().count += 1;
//             if state.inner().name == "".to_string() {
//                 state.inner_mut().name = "Zachary".to_string();
//             }

//             Ok(format!("Hello World, and {:?}: {} times", state.inner().name, state.inner().count))
//         }

//         let mut lock_state = self.0.lock().unwrap();
//         match endpoint_call(&mut *lock_state) {
//             Ok(data) => Response::from(data),
//             Err(code) => Response::from(code),
//         }
//     }
// }

// #[route("/", methods=[get, post])]
// fn message(_cx: Context) -> Result<&'static str> {
//     // PERF: Support for return type of Responder.
//     // templating with HandleBars and Tera
//     // Macro based `rsx` / templating
//     Ok(r#"<html>
//         <head>
//             <title>Home</title>
//         </head>
//         <body>
//             <h1>Hello World</h1>
//             <ul>
//                 <li>Welcome</li>
//                 <li>to</li>
//                 <li>LaunchPad</li>
//             </ul>
//         </body>
//     </html>"#)
// }

// macro_rules! rsx {
//     ($($markup:tt)*) => {
//         Element
//     };
// }

// #[component]
// fn button(cx: Option<String>, children: Vec<Element>, name: &str) -> Element {
//     rsx! {
//         <button @click={|e| cx.find("#message>div").toggle("d-none")}>
//             {name}
//             {children}
//         <button/>
//     }
// }
