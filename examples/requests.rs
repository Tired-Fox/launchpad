extern crate wayfinder;

use serde::{Deserialize, Serialize};
use wayfinder::{
    prelude::*,
    request::{Body, Query},
    response::{HTML, JSON},
    Server,
};

/// Wayfinder suppports uri captures. These are parts of a path that match a pattern
/// and are captured into variables. These variables can then be optionally used as parameters
/// to the endpoint. When they are used as parameters String::parse::<T>() is used to cast to the
/// matching parameters type. The name of the parameter must match the uri capture exactly to be
/// matched. `:name` will capture a single part of the path into the name. `:...name` is a catch
/// all capture and will get all following parts of the path into name. This can be the rest of
/// the uri, or until the next static part of the uri. A single capture can not follow a catch all
/// capture.
///
/// Captures can be wrapped in Option or Result to prevent automatic failure and 500 error when the
/// capture is attempting to be parsed to it's respective type. Option will give None if it fails
/// while Result will give a tuple, `(code, message)`, where it can be returned in a Result
/// response or it can be furthure processed.
#[get("/api/:firstname/:lastname/from/:...path")]
pub fn uri_capture(
    firstname: String,
    lastname: Option<String>,
    path: Result<String>,
) -> HTML<String> {
    html! {
        <h1>{firstname}" "{lastname.unwrap_or("<None>".to_string())}": "<code>{path.unwrap()}</code></h1>
    }
}

/// Wayfinder support automatic parsing of the uri query as a parameter. If a parameter
/// is set to be `Query` it will parse the uri query into it's generic type. This can be a
/// String, or it can be any Deserializable object supported by serde_qs. The result is wrapped in
/// a Query struct but can be destructured right away. If the query is optional it can be wrapped
/// in an Option enum and the result of the parse it converted to an option instead of being
/// unwrapped. The parameter can to also be wrapped in a Result. This will capture the error code and message
/// from parsing the query. If the query is not wrapped and the parse failes the endpoint automatically
/// responds with a 500 interal server error.
///
/// See `optional_query` endpoint for more ways to use the Query parameter.
#[get("/api/query")]
pub fn query(Query(q): Query<String>) -> HTML<String> {
    html! {
        <h4>"Query: "{ q }</h4>
    }
}

/// Used for the optional_query endpoint
/// Default is required for debug mode to display a rich error page.
#[derive(Deserialize, Serialize, Default)]
struct UserQuery {
    name: String,
}
/// This endpoints shows additional ways of using the Query parameter. See `query` endpoint for
/// base usage.
#[get("/api/optional-query")]
pub fn optional_query(q: Option<Query<UserQuery>>) -> Result<JSON<UserQuery>> {
    match q {
        Some(Query(q)) => response!(JSON(q)),
        None => {
            response!(500, "Invalid or missing query")
        }
    }
}

/// Wayfinder supports parsing the request body in a parameter.
/// The type of the body can be string, which retains the body as a raw string,
/// or as a Deserialize object. This can be a serde Deserialize struct, and it will use serde_json
/// by default, or it can be any serde_plain supported object. This means things like u32 or Enums
/// can also be parsed from the body.
///
/// Body is very similar to Query. It can be marked as optional. To allow for missing or invalid
/// parsing of the body to not result in an error response. See `optional_body` endpoint to see
/// more of what body can do. Also like Query, body also allows for the parameter to be a result.
/// This will capture the error code and message from parsing the body.
#[post("/api/body")]
pub fn bbody(Body(b): Body<String>) -> HTML<String> {
    html! {
        <h4>"Body"</h4>
        <pre>{b}</pre>
    }
}

/// This endpoint is to show additional features of Body.
/// See `body` for base use case.
/// This one specifically shows how result can be used to capture any errors
/// that occur while parsing the body.
#[post("/api/optional-body")]
pub fn optional_body(b: Result<Body<u32>>) -> Result<HTML<String>> {
    // Map the successfully parsed body into an HTML response.
    // Return the error from parsing the body if there is one.
    b.map(|Body(num)| {
        html! {
            <h4>"Body"</h4>
            <pre>{num}</pre>
        }
    })
}

#[get("/")]
fn home() -> HTML<String> {
    html! {
        <script>
         "
            function body_request() {
               fetch(
                'http://localhost:3000/api/body',
                { method: 'POST', body: 'Hello, world!' }
               )
                 .then(async response => {
                    const result = document.getElementById('result');
                    if (result) {
                      let text = await response.text();
                      result.innerHTML = text;
                    }
                 })
                 .catch(reason => console.error(reason));
            }
            function optional_body_request() {
               fetch(
                'http://localhost:3000/api/optional-body',
                { method: 'POST', body: 32 }
               )
                 .then(async response => {
                    const result = document.getElementById('result');
                    if (result) {
                      let text = await response.text();
                      result.innerHTML = text;
                    }
                 })
                 .catch(reason => console.error(reason));
            }
         "
        </script>
        <button onclick="body_request()">"Body Request"</button>
        <button onclick="optional_body_request()">"Optional Body Request"</button>
        <div id="result"></div>
    }
}

/// Run `cargo run --example requests`
/// Note: All valid parameters to an endpoint can be made optional. This allows for failed
/// parameter parsing to be None instead of automatically returning 500 internal server error.
#[wayfinder::main]
async fn main() {
    Server::new()
        .route(home)
        .route(uri_capture)
        .routes(group![query, optional_query])
        .routes(group![bbody, optional_body])
        .serve(3000)
        .await
}
