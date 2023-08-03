extern crate wayfinder;
mod routes;

// Routes are defined elsewhere
use routes::{error, files, home, json, redirect, uri_capture};

// Everything needed to create and start the server
use wayfinder::{prelude::*, Server};

// Allows for tokio::main and result response
#[wayfinder::main]
async fn main() {
    Server::new()
        .assets("examples/web/")
        // Unique uri with captures
        .route(uri_capture)
        // Standard endpoint
        .route(home)
        // All endpoints covering how errors work
        .routes(group![redirect, error::server])
        // All endpoints targeting how JSON works
        .routes(group![json::json_string, json::hello_world])
        // All endpoints targeting how file responses works
        .routes(group![
            files::html_file,
            files::text_file,
            files::text_to_html_file,
            files::json_file,
            files::text_to_json_file
        ])
        // Error handler
        .catch(error::not_found)
        .serve(3000)
        .await
}
