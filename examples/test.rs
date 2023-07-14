extern crate launchpad;

use launchpad::{
    prelude::*,
    response::{File, HTML, JSON},
    Data, Server,
};

#[tokio::main]
async fn main() {
    Server::new(([127, 0, 0, 1], 3000))
        .router(routes![home, data])
        .serve()
        .await;
}

#[get("/")]
fn home() -> Result<HTML<File>> {
    HTML::of(File::from("index.html"))
}

#[post("/api/name/<firstname>/<lastname>/")]
fn data(data: Data<HomeData>, firstname: String, lastname: String) -> Result<JSON<User>> {
    JSON::of(User {
        firstname,
        lastname,
        age: data.get_ref().age,
        male: data.get_ref().male,
    })
    // Error::new(500, "Testing user errors")
}

#[derive(Debug, serde::Serialize)]
struct User {
    firstname: String,
    lastname: String,
    age: u16,
    male: bool,
}

#[derive(Default, Debug, serde::Deserialize)]
struct HomeData {
    age: u16,
    male: bool,
}
