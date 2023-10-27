Rust based web design heavily inspired by [Axum](https://docs.rs/axum/latest/axum/)

# High-Level Features

- Routing to async handlers
- Request extractors in handler parameters
- Easy to use objects
- As close to bare rust as possible
- Minimal and optional macros

# Compatibility
Tela is designed to use [tokio](https://docs.rs/tokio/1.29.1/tokio/index.html) and [hyper](https://docs.rs/hyper/0.14.27/hyper/index.html). Some tokio and hyper objects are re-exported from tela, but if more functionality
is desired then those crates will have to be imported by the user.

# Example

A simple tela "Hello, World!".
```rust
use tela::{server::{Server, Router, methods::get, Socket}};

#[tela::main]
async fn main() {
    Server::builder()
        .on_bind(|addr| println!("Serving at {}", addr))
        .serve(
            Socket::Local(3000),
            Router::builder()
                .route("/", get(|| async { "Hello, world!" }))
        ).await;
}
```

Note: `#[tokio::main]` can be used instead of `#[tela::main]` except it would require tokio to be added as a dependency with the
`macros` feature. `#[tela::main]` acts like the `#[tokio::main]` macro but doesn't require tokio to be added as an additional dependency.

# Routing
[Router](crate::server::Router) is the heart of handling hyper based requests. The struct handles what path goes to which static asset or handler.

```rust
use tela::server::{Router, methods};

fn main() {
    // A router that serves static assets and two routes
    let router = Router::builder()
        .assets(("/", "assets/public/"))
        .route("/", methods::get(home))
        .route("/edit", methods::post(home));
}

// Base handlers that are called, but produce nothing
async fn home() {}
async fn edit() {}
```
See [Router](crate::server::Router) for more details on routing.

# Handler
"Handler" in tela means any async function that accepts 0 to 15 extractors and returns something that can be
turned [into a response](crate::response).

Handlers are where most of your app logic is located.

See [handler](crate::server::router::handler) for more details on handlers.

# Extractors

Extractors are anything that implement [FromRequest](crate::request::FromRequest) or [FromRequestParts](crate::request::FromRequestParts).
This allows you to break down and only take parts of the request into the handler that you want.

```rust
use tela::extract::{Json, Query};
use tela::json::Value;

// Buffers the hyper::Request body and deserializes it a serde_json::Value.
// This consumes the request so this must be the last argument.
async fn json(Json(payload): Json<Value>) {}

// `Query` give the query parameters and deserializes them using `serde_qs`.
async fn query(Query(params): Query<String>) {}
```

See [extract](crate::extract) for more information on extractors.

# Response
Anything that implements [IntoResponse](crate::response::IntoResponse) can be returned from any handler.

```rust
use tela::response::{Json, Html};
use serde::{Serialize, Deserialize};

async fn plain_text() -> &'static str {
    "hello, world!"
}

async fn html() -> Html<&'static str> {
    Html("<h1>Hello, world</h1>")
}

// A struct that can be serialized or deserialized which is needed for the Json struct.
#[derive(Serialize)]
struct Data {
    name: &'static str,
    count: i32
}

// `Json` gives a Content-Type of `application/json` and works with anything that implements
// serde::Serialize
async fn json() -> Json<Data> {
    Json(Data { name: "Tela", count: 3 })
}
```

See [response](crate::response) for more details on building responses

# Using State
With web applications it may make sense to have some sort of application state that is shared between handlers.
This could be anything from database connections, templating engine instances, or any other state that needs to be persisted between handlers.

Right now tela primarily uses the `State` extractor to manage application state with other methods potentially coming later.

```rust
use tela::server::{Router, methods::get, State};

#[derive(Clone)]
struct AppState {
    // ...
}

fn main() {
    let _ = Router::builder()
        .route("/", get(handler))
        .state(state);
}

async fn handler(State(app_state): State<AppState>) {}
```

See [State](crate::server::State) for more information on how to set up and use the State extractor.

# Templating

Returning some sort of html can be extremely useful for serving a web application. With that thought, tela provides a html
templating macro that constructs simple and lightweight objects. This macro does not need to be used and something else like
[html-to-string-macro](https://docs.rs/html-to-string-macro/latest/html_to_string_macro/) could be used instead.

```rust
use tela::html;

fn component(props: html::Props) -> html::Element {
    html::new!(<p>"Component"</p>)
}

async fn async_component(props: html::Props) -> html::Element {
    html::new!(<p>"Async Component"</p>)
}

async fn handler() -> html::Element {
    let data = ["one", "two", "three"];
    let name = "Tela";
    html::new! {
        <!-- "Supports comments" -->
        <h1>"Regular html syntax"</h1>
        <p>"Injections: "{name}</p>
        
        <component />
        <async-component await />
        
        <!-- "Loops" -->
        <for let:data>
            {|item: &'static str| {
                html::new!(<p>{item}</p>)    
            }}
        </for>
        <!-- "Async loops" -->
        <for let:data async>
            {|item: &'static str| async move {
                html::new!(<p>{item}</p>)    
            }}
        </for>
    }
}

```

See [html](crate::html) for more details on tela's html templating.

# Required Dependencies

tela strives to keep the dependencies to a minimum to keep the library simple and easy to use. However, there are limitations
and some dependencies are needed.

```toml
[dependencies]
serde = { version = "<latest-version>", features=["derive"] }
```

# Examples

The tela repo contains some [simple examples](https://github.com/Tired-Fox/tela/tree/main/examples) that show how to potentially use the library.

# Feature Flags
tela uses a few feature flags for optional functionality that a user may want, but most will not.
The following features are available:

| Name | Description | Default? |
|:----|:-----|:----|
| comments | Turn on templating comments so the render in the final output | No |