extern crate wayfinder;

use serde::{Deserialize, Serialize};
use wayfinder::{
    prelude::*,
    request::{Body, Query},
    response::{Redirect, HTML, JSON},
    Server,
};

#[derive(Deserialize, Debug, Serialize)]
pub struct UserQuery {
    name: String,
}

#[get("/hello-world")]
pub fn hello_world(query: Option<Query<UserQuery>>) -> (u16, JSON<UserQuery>) {
    // Can respond with a custom response code
    // this is returned outright without catching error codes. Redirect codes are still caught
    match query {
        Some(Query(query)) => (201, JSON(query)),
        _ => (
            203,
            JSON(UserQuery {
                name: String::new(),
            }),
        ),
    }
}

#[get("/api/:firstname/:lastname/:age")]
pub fn uri_capture(firstname: String, lastname: String, age: u32) -> HTML<String> {
    html! {
        <h1>{firstname}" "{lastname}": age "{age}</h1>
    }
}

#[get("/redirect")]
pub fn redirect() -> Redirect<303> {
    // Redirect defaults to 302, but can be 301, 302, 303, 307, or 308
    Redirect("/".to_string())
}

#[get("/error")]
pub fn error() -> Result<HTML<String>> {
    // Can return a result of Ok(T) or Err((code, message))
    // response! shortcuts and wraps what is inside making the syntax simpler
    response!(500, "Custom user error")
}

/**
 * The home route will display what the uri query was. If there are no query then
 * there is a 500 error response
 */
#[get("/")]
pub fn home(Query(query): Query<String>, Body(body): Body<String>) -> HTML<String> {
    // Standard endpoint that has a html response
    println!("{:?}", body);

    html! {
       <p>"query: "{query}</p>
    }
}

/**
 * Not Found error catch documentation
 * This will be put right below a doc about what the function handles
 */
#[catch(404)]
pub fn not_found(code: u16, message: String, reason: String) -> HTML<String> {
    // 404 Error handler. Must return valid response. Neither (code, data) or Result<data>
    // will work.
    html! {
        <h1>{code}" "{message}</h1>
        <p>{reason}</p>
    }
}

#[wayfinder::main]
async fn main() {
    Server::new()
        .routes(group![hello_world, home, error, redirect])
        .route(uri_capture)
        .catch(not_found)
        .serve(3000)
        .await
}
