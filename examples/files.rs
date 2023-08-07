extern crate wayfinder;

use serde::{Deserialize, Serialize};
use wayfinder::{
    prelude::*,
    response::{File, HTML, JSON},
    Server,
};

#[get("/html-file")]
pub fn html_file() -> File<&'static str> {
    File("examples/assets/index.html")
}

#[get("/text-file")]
pub fn text_file() -> File<&'static str> {
    File("examples/assets/index.txt")
}

#[get("/text-to-html-file")]
pub fn text_to_html_file() -> HTML<File<&'static str>> {
    HTML(File("examples/assets/index.txt"))
}

#[get("/json-file")]
pub fn json_file() -> File<&'static str> {
    File("examples/assets/sample.json")
}

#[derive(Deserialize, Serialize)]
pub struct User {
    name: String,
    age: u16,
    description: String,
}

#[get("/text-to-json-file")]
pub fn text_to_json_file() -> Result<JSON<User>> {
    JSON::from_file(File("examples/assets/sample.txt"))
}

/// Run `cargo run --example files`
///
/// Files are run from relative path of static files
///
/// Files can be returned outright and the content type is inferred from the files extension.
/// Otherwise, if you want to ensure a certain content type, a return like `JSON` or `HTML` can
/// be used to wrap the File.
///
/// The `group!` macro allows for the `routes` and `catches` builder methods to be used on the
/// server. It allows for a list of endpoints or error handlers to be grouped together and added to
/// the router at the same time. If there is a lot going on in the main method this can be used in the returns
/// of helper methods in other modules that can be called in the server. For example if there the
/// file endpoints defined below were moved to a different module the setup can look like this. As
/// long as it is a vec of Arc<dyn Endpoint> or Arc<dyn Catch> then the way of adding routes
/// doesn't matter.
///
/// ```rust
/// fn file_group() -> Vec<Arc<dyn Endpoint>> {
///     group![
///         html_file,
///         text_file,
///         text_to_html_file,
///         json_file,
///         text_to_json_file
///     ]
/// }
///
/// #[wayfinder::main]
/// async fn main() {
///     Server::new()
///         .routes(file_group())
///         .serve(3000)
///         .await
/// }
/// ```
#[wayfinder::main]
async fn main() {
    Server::new()
        .routes(group![
            html_file,
            text_file,
            text_to_html_file,
            json_file,
            text_to_json_file
        ])
        .serve(3000)
        .await
}
