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

[typed-html](https://crates.io/crates/typed-html/0.2.2) for html macro inspiration
[html-to-string-macro](https://docs.rs/html-to-string-macro/latest/src/html_to_string_macro/lib.rs.html#96-105) for inspiration

Plan for RTX (JSX).
```rust
  rtx! {
    <html lang="en">
      <head>
        <title>Something</title>
      </head>
      ... etc
    </html>
  }
```

With components able to do things like
```rust
#[component]
fn Sample(cx: Context) -> Element {
  let p1 = "prop 1";
  let p3 = "prop 3";

  rtx! {
    <Title>Inject into head</Title>
    <div>
      <OtherComponent p1 p2=p3 />
    </div>
  }
}
```

Has components like:
```rust
rtx!{
  <Router>
    <Route href="/sample">
      <Sample />
    </Route>
    <Route href="/">
      <Home />
    </Route>
  </Router>
}
```

Uses the above http server handler but now has the ability to serve wasm based
components.

<!-- Footer Badges --!>

<br>
<div align="center">
  <img src="assets/badges/made_with_rust.svg" alt="Made with rust"/>
  <img src="assets/badges/built_with_love.svg" alt="Built with love"/>
</div>

<!-- End Footer -->
