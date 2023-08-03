use serde::{Deserialize, Serialize};
use wayfinder::{
    prelude::*,
    response::{File, HTML, JSON},
};

#[get("/html-file")]
pub fn html_file() -> File<&'static str> {
    File("examples/web/index.html")
}

#[get("/text-file")]
pub fn text_file() -> File<&'static str> {
    File("examples/web/index.txt")
}

#[get("/text-to-html-file")]
pub fn text_to_html_file() -> HTML<File<&'static str>> {
    HTML(File("examples/web/index.txt"))
}

#[get("/json-file")]
pub fn json_file() -> File<&'static str> {
    File("examples/web/sample.json")
}

#[derive(Deserialize, Serialize)]
pub struct User {
    name: String,
    age: u16,
    description: String,
}

#[get("/text-to-json-file")]
pub fn text_to_json_file() -> Result<JSON<User>> {
    JSON::from_file(File("examples/web/sample.txt"))
}
