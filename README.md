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
#[get("/")]
fn home() -> Result<&'static str> {
  Ok("Hello, world!")
}
```

```rust
#[post("/login/<username>/<age: int>")]
fn data(login: &str, age: i32) -> Result<HTML<&'static str>> {
  HTML::of("<h1>Hello, world!</h1>")
}
```

```rust
#[catch(404)]
fn not_found(code: u16, message: String) -> String {
  format!("<h1>{} {}</h1>", code, message)
}
```

Run an app like so.
```rust
use launchpad::{rts, Server};

#[tokio::main]
asyn fn main() {
  Server::new()
      .router(rts!{
          [ home, data ],
          catch { not_found }
      })
      .serve(3000)
      .await;
}
```

Uses the above http server handler but now has the ability to serve wasm based
components.

## Goals for Rewrite:
- [ ] Extractors
- [ ] From / Into type of traits
  - Complex param traits for optionals and parsing specific parts of request
  - Response to allow for easy and complex return types close to native rust that are easy to use and give the most feedback and freedom as possible
- [ ] From and Into traits for most objects
  - The idea would be around things like `axum` extractor's
  - User can just define a parameter and it will inject/convert
  - Option varient is also viable and can be null if it fials
  - Extractors also allow for finding out the content type
    - JSON extractor will return an `application/json` content type
    - HTML extractor will return a `text/html` content type
    - etc...
- [ ] Response is a Result that implements ToResponse
- [ ] Failed response is a status code and optional user defined message
- [ ] Parameter macros, method macros, and constructor macros
- [ ] Built it timeout, throtteling, etc... with `Tower`
- [ ] HTTP/1 and HTTP/2 Support

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

- [axum extractors](https://github.com/search?q=repo%3Atokio-rs%2Faxum+extractor&type=code)

**Structure**
- Router
  - Filtering
    - URL
    - Props
    - Response
  - Defaults
  - Call and Convert
- Macros
  - router - similar to what is done now but more organized and specific
  - request wrapper - similar to what is written now
  - props - [leptos](https://leptos-rs.github.io/leptos/view/03_components.html?highlight=%23%5Bprops#into-props)

**Branding to Spider Inspiration**
- `Tangle` or `Cobweb`
- `Orb-Weaver`
- `Black-Widow`

<!-- Footer Badges --!>

<br>
<div align="center">
  <img src="assets/badges/made_with_rust.svg" alt="Made with rust"/>
  <img src="assets/badges/built_with_love.svg" alt="Built with love"/>
</div>

<!-- End Footer -->
