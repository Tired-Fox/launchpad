extern crate wayfinder;

use serde::{Deserialize, Serialize};
use wayfinder::{
    prelude::*,
    request::{Body, Query},
    response::{HTML, JSON},
    Server,
};

#[derive(Deserialize, Debug, Serialize)]
pub struct UserQuery {
    name: String,
}

#[get("/hello-world")]
pub fn hello_world(query: Option<Query<UserQuery>>) -> JSON<UserQuery> {
    match query {
        Some(Query(query)) => JSON(query),
        _ => JSON(UserQuery {
            name: String::new(),
        }),
    }
}

#[get("/")]
pub fn home(Query(query): Query<String>, Body(body): Body<String>) -> Result<HTML<String>> {
    println!("{:?}", body);

    response!(
        // html! {
        //    <p>"query: "{query}</p>
        // }
        500,
        "User defined error"
    )
}

#[catch(404)]
pub fn not_found(code: u16, message: String, reason: String) -> HTML<String> {
    html! {
        <h1>{code}" "{message}</h1>
        <p>{reason}</p>
    }
}

#[wayfinder::main]
async fn main() {
    Server::new()
        .route(hello_world)
        .route(home)
        .catch(not_found)
        .serve(3000)
        .await
}
