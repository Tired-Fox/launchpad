extern crate launchpad;
use launchpad::{prelude::*, response::JSON, request::{State, Data, Query}, Result};

#[post("/api/name/<firstname>/<lastname>/")]
pub fn data(
    state: &mut State<HomeState>,
    firstname: String,
    lastname: String,
    data: Data<HomeData>,
    query: Query<UserQuery>
) -> Result<JSON<User>> {
    println!("{:?}", query.get_ref());

    if state.get_ref().name == String::new() {
        state.get_ref_mut().name = String::from("Zachary");
    }
    println!("{}", state.get_ref().name);

    JSON::of(User {
        firstname,
        lastname,
        age: data.get_ref().age,
        male: data.get_ref().male,
    })
    // Error::new(500, "Testing user errors")
}

#[derive(Debug, Default, serde::Deserialize)]
struct UserQuery {
    name: String,
    age: u16
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct User {
    firstname: String,
    lastname: String,
    age: u16,
    male: bool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct HomeData {
    age: u16,
    male: bool,
}
#[derive(Debug, Default)]
pub struct HomeState {
    name: String
}
