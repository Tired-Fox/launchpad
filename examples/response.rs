extern crate tela;

use tela::{
    prelude::*,
    response::{Raw, Redirect, HTML, JSON},
    Server, StatusCode,
};

/// tela supports redirect responses. Just define which
/// redirect code is desired, defaults to 302, and return a Redirect object
/// with the redirect location.
#[get("/redirect")]
pub fn redirect() -> Redirect<303> {
    // Redirect defaults to 302, but can be 301, 302, 303, 307, or 308
    Redirect::to("/")
}

/// tela supports endpoints that could return a error response. This response
/// is captured and potentially handled by an error handler. The user may define a custom
/// error message which is passed as `reason` to a handler and used in the `tela-Reason`
/// header in the response.
#[get("/error")]
pub fn server_error() -> Result<String> {
    // Can return a result of Ok(T) or Err((code, message))
    // response! shortcuts and wraps what is inside making the syntax simpler
    response!(StatusCode::InternalServerError, "Custom user error")
    // == response!(500, "Custom user error")
    // == Err((500, "Custom user error"))
}

/// tela has built in json support using serde_json.
/// Using the JSON object a shapeless json can be returned along
/// with serializable structs.
#[get("/json")]
pub fn json() -> JSON<Raw> {
    // json! macro is from serde_json and constructs a serde_json::Value object.
    JSON(json!({ "message": "Hello, world!" }))
}

/// tela has HTML response support. The response
/// can be of any type that can be converted to a string.
/// The `html!` macro returns a `HTML<String>` and allows for a markup style
/// format for variable injection.
#[get("/")]
pub fn html() -> HTML<String> {
    let message = "Home page";
    html! {
        <h1>{message}</h1>
        <p>"created from a "<code>"html!"</code>" macro"</p>
    }
}

#[tela::main]
async fn main() {
    Server::new()
        // `/redirect`
        .route(redirect)
        // `/error`
        .route(server_error)
        // `/json`
        .route(json)
        // `/`
        .route(html)
        .serve(3000)
        .await
}
