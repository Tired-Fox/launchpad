extern crate launchpad;
use launchpad::{prelude::*, response::{HTML, File}, Result, Error};

pub mod api;

#[get("/")]
pub fn index() -> Result<HTML<File>> {
    HTML::of(File::from("index.html"))
}

#[get("/error")]
pub fn error_page() -> Result<HTML<&'static str>> {
    Error::of(500, "Custom user error response")
}

/// Catch all endpoint for `404` not found error page
/// Handles all unkown error pages 
/**
 * Multiline comment
 * here
 */
#[catch(404)]
pub fn not_found(code: u16, message: String) -> String {
    format!(r#"
<html>
    <head>
        <title>{0} {1}</title>
    </head>
    <body>
        <h1>{0} {1}</h1>
        <p>Oops it looks like the page you are trying to reach doesn't exist</p>
        <a href="/">Back to Home</a>
    </body>
</html>"#,
    code,
    message)
}
