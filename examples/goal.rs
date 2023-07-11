extern crate launchpad;

use launchpad::{prelude::*, Server, State};

#[tokio::main]
async fn main() {
    Server::new(([127, 0, 0, 1], 3000))
        .router(routes![world, home])
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

    Ok(format!(
        "Hello World, and {}: {} times",
        state.inner().name,
        state.inner().count
    ))
}

#[request("/", methods=[get, post])]
fn home() -> Result<&'static str> {
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

#[post("/")]
fn data() -> Result<&'static str> {
    Ok("Home")
}

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
