# launchpad 

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

Rust based web design :smile:

Construct endpoints or error handlers like so.

```rust
// This is a text/plain response
#[get("/")]
fn home() -> &'static str {
  "Hello, world!"
}
```

```rust
// This is a text/html response that could fail and the error should be either
// given to the appropriate handler or returned as is.
#[post("/login/<username>/<age: int>")]
fn data(login: &str, age: i32) -> Result<HTML<String>> {
  response!(html!(<h1>"Hello, world!"</h1>))
}
```

```rust
// Catches any error that is 404 comming from another endpoint
// soon this will be for all 404 errors that are thrown
#[catch(404)]
fn not_found(code: u16, message: String) -> HTML<String> {
  html!(<h1>{code}" "{message}</h1>)
}
```

Run an app like so.
```rust
use wayfinder::{prelude::*, Server};

#[wayfinder::main]
asyn fn main() {
  Server::new()
      .routes(group![home, data])
      .catch(not_found)
      .serve(3000)
      .await
}
```

## TODO:
- [ ] Built it timeout, throtteling, etc... with `Tower`
- [ ] HTTP/1 and HTTP/2 Support (Currently only HTTP/1)

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

- [typed-html](https://crates.io/crates/typed-html/0.2.2) for html macro inspiration and [html-to-string-macro](https://docs.rs/html-to-string-macro/latest/src/html_to_string_macro/lib.rs.html#96-105) for html responses. 

<!-- Footer Badges --!>

<br>
<div align="center">
  <img src="assets/badges/made_with_rust.svg" alt="Made with rust"/>
  <img src="assets/badges/built_with_love.svg" alt="Built with love"/>
</div>

<!-- End Footer -->
