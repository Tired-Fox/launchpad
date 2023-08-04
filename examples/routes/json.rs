use serde::{Deserialize, Serialize};
use wayfinder::{
    prelude::*,
    request::Query,
    response::{Raw, JSON},
    StatusCode,
};

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct UserQuery {
    name: String,
}

#[get("/json-string")]
pub fn json_string() -> JSON<Raw> {
    JSON(json!({"name": "zachary", "age": 23}))
}

#[get("/hello-world")]
pub fn hello_world(query: Option<Query<UserQuery>>) -> (StatusCode, JSON<UserQuery>) {
    // Can respond with a custom response code
    // this is returned outright without catching error codes. Redirect codes are still caught
    match query {
        Some(Query(query)) => (StatusCode::Created, JSON(query)),
        _ => (
            StatusCode::ImATeapot,
            JSON(UserQuery {
                name: String::new(),
            }),
        ),
    }
}
