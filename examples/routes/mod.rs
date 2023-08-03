use wayfinder::{
    prelude::*,
    request::{Body, Query},
    response::{Redirect, HTML},
};

pub mod error;
pub mod files;
pub mod json;

#[get("/api/:firstname/:lastname/:...path")]
pub fn uri_capture(firstname: String, lastname: String, path: String) -> HTML<String> {
    html! {
        <h1>{firstname}" "{lastname}": path "{path}</h1>
    }
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

#[get("/redirect")]
pub fn redirect() -> Redirect<303> {
    // Redirect defaults to 302, but can be 301, 302, 303, 307, or 308
    Redirect::to("/")
}
