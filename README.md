# Tela 

<!-- Header Badges -->

<div align="center">
  
<img src="assets/badges/version.svg" alt="Version"/>
<a href="https://github.com/Tired-Fox/launchpad/releases" alt="Release"><img src="https://img.shields.io/github/v/release/tired-fox/launchpad.svg?style=flat-square&color=9cf"/></a>
<a href="https://github.com/Tired-Fox/launchpad/blob/main/LICENSE" alt="License"><img src="assets/badges/license.svg"/></a>
<br>
<img src="assets/badges/maintained.svg" alt="Maintained"/>
<img src="assets/badges/tests.svg" alt="Tests"/>
  
</div>

<!-- End Header -->
___

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
"Handler" in tela means any async function that accepts 0 to 15 

# Extractors
# Response
# Using State
# Required Dependencies
# Examples
# Feature Flags


```rust
// This is a text/plain response
async fn home() -> &'static str {
  "Hello, world!"
}
```

```rust
use tela::{prelude::*, html};
// This is a text/html response that could fail and the error should be either
// given to the appropriate handler or returned as is.
async fn data() -> Result<Html<String>, Error> {
  Ok(html::into!(<h1>"Hello, world!"</h1>))
}
```

```rust
use tela::{prelude::*, Json, json::{self, Value}};
// Endpoint that returns json with a custom HTTP code.
fn get_data() -> (u16, Json<Value>) {
  (203, json::into!({"name": "Tela"}))
}
```

```rust
use tela::{prelude::*, Json, json::Value, Query, request::Body};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
  name: String,
}

// The query and body can automatically be extracted from the request in the parameters.
// Just use `Body` and `Query`. If an extraction or a uri capture could be missing or you don't want
// Tela throwing a 500 error automatically, you can wrap the parameters type in an `Option`.
// Also note that the order of the parameters are not important.
fn get_user(query: Option<Query<User>>, username: String, body: i32 /* Last parameter is assumed to consume the body */) -> Json<Value> {
  let username = match query {
    Some(User{name}) => name,
    None => String::new()
  };

  json::into!({"name": username, "age": body})
}
```

Run an app like so.
```rust
use tela::{prelude::*, server::{Server, Router, methods::get, Socket}};

#[tela::main]
async fn main() {
  Server::builder()
      .serve(
          Socket::Local(3000),
          Router::builder()
              .route("/", get(home))
              .assets(("/", "assets/public/"))
      )
      .await;
}

async fn home() -> &'static str {
    "Hello, world!"
}
```

With state.
```rust
use tela::{prelude::*, server::{Server, Router, methods::get, Socket}};

#[derive(clone)]
struct AppState {
    name: &'static str
}

#[tela::main]
async fn main() {
    let state = AppState { name: "Tela" };
    Server::builder()
        .serve(
            Socket::Local(3000),
            Router::builder()
                .route("/", get(home))
                .assets(("/", "assets/public/"))
        )
       .await;
}

async fn home() -> &'static str {
    "Hello, world!"
}
```

<!-- Footer Badges -->

<br>
<div align="center">
  <img src="assets/badges/made_with_rust.svg" alt="Made with rust"/>
  <img src="assets/badges/built_with_love.svg" alt="Built with love"/>
</div>

<!-- End Footer -->
