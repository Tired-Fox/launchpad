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

> *_Notice:_* The library is currently going through a rewrite to use less macros. The macro use will be redirected toward templating and helers but will be comletely optional in use. 

___

Rust based web design :smile:

Construct endpoints or error handlers like so.

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

**Inspiration**
- [Axum](https://github.com/tokio-rs/axum)
- [Actix](https://github.com/actix/actix-web)
- [Warp](https://github.com/seanmonstar/warp)
- [Rocket](https://rocket.rs/)

**Tools**
- [Tokio](https://tokio.rs/)
- [Hyper](https://hyper.rs/) - Focus on `1.0 release`
- [Tower](https://github.com/tower-rs/tower)

- [proc-macro-errors](https://docs.rs/proc-macro-error/latest/proc_macro_error/)
- [proc-macro2](https://docs.rs/proc-macro2/latest/proc_macro2/)
- [quote](https://docs.rs/quote/latest/quote/)
- [syn](https://docs.rs/syn/latest/syn/)

- [typed-html](https://crates.io/crates/typed-html/0.2.2) and [html-to-string-macro](https://docs.rs/html-to-string-macro/latest/src/html_to_string_macro/lib.rs.html#96-105) for html macro inspiration . 

<!-- Footer Badges -->

<br>
<div align="center">
  <img src="assets/badges/made_with_rust.svg" alt="Made with rust"/>
  <img src="assets/badges/built_with_love.svg" alt="Built with love"/>
</div>

<!-- End Footer -->
