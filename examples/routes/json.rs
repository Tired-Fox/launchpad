use serde::{Deserialize, Serialize};
use wayfinder::{
    prelude::*,
    request::Query,
    response::{Raw, JSON},
};

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct UserQuery {
    name: String,
}

#[get("/json-string")]
pub fn json_string() -> JSON<Raw> {
    JSON(r#"{"name": "zachary", "age": 23}"#.into())
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
