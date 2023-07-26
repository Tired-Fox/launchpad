extern crate launchpad;

use launchpad::{
    prelude::*,
    router::{
        request::{Content, Query, State},
        response::JSON,
    },
};

#[post("/api/plain")]
pub fn plain(
    // Support for primitives and Enums. This consumes the entire body and must be text/plain
    content: Content<String>,
) -> Result<String> {
    Ok(content.get_ref().clone())
}

#[post("/api/name/<firstname>/<lastname>/")]
pub fn data(
    state: &mut State<HomeState>,
    firstname: String,
    lastname: String,
    content: Content<HomeData>,
    query: Query<UserQuery>,
) -> Result<JSON<User>> {
    println!(
        "UserQuery: {}, {}",
        query.get_ref().name,
        query.get_ref().age
    );

    if state.get_ref().name == String::new() {
        state.get_ref_mut().name = String::from("Zachary");
    }
    println!("HomeState: {}", state.get_ref().name);

    // Serialize from file
    // use launchpad::response::File;
    // JSON::parse(File::from("user.json"))

    // Deserialize from string into struct then serialize into bytes
    // This insures that the data is valid before returning
    // JSON::parse(format!(r#"{{
    //    "firstname": "{}",
    //    "lastname": "{}",
    //    "age": {},
    //    "male": {}
    // }}"#,
    //     firstname,
    //     lastname,
    //     data.get_ref().age,
    //     data.get_ref().male
    // ).as_str())

    // From a serializable struct
    JSON::ok(User {
        firstname,
        lastname,
        age: content.get_ref().age,
        male: content.get_ref().male,
    })
}

#[derive(Debug, Default, serde::Deserialize)]
struct UserQuery {
    name: String,
    age: u16,
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
    name: String,
}
