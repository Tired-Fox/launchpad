extern crate launchpad;
use launchpad::{
    prelude::*,
    router::response::{File, HTML},
};

pub mod api;

#[get("/")]
pub fn index() -> Result<File> {
    File::ok("index.html")
}

#[get("/error")]
pub fn error_page() -> Result<HTML<&'static str>> {
    HTML::err(500, "Custom user error response")
}

/// Catch all endpoint for `404` not found error page
/// Handles all unkown error pages
/**
 * Multiline comment
 * here
 */
#[catch(404)]
pub fn not_found(code: u16, message: String) -> String {
    html! {
        <html>
            <head>
                <title>{code}" "{message.clone()}</title>
            </head>
            <body>
                <h1>{code}" "{message}</h1>
                <p>"Oops it looks like the page you are trying to reach doesn't exist"</p>
                <a href="/">"Back to Home"</a>
            </body>
        </html>
    }
}

#[catch]
pub fn unexpected(code: u16, message: String) -> String {
    html! {
        <html>
            <head>
                <title>{code}" "{message.clone()}</title>
            </head>
            <body>
                <h1>{code} {message}</h1>
                <p>"An error occured, please try again."</p>
                <a href="/">"Back to Home"</a>
            </body>
        </html>
    }
}
