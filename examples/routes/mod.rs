extern crate launchpad;
use launchpad::{prelude::*, response::{HTML, File}, Result, Error};

pub mod api;

#[get("/")]
pub fn index() -> Result<HTML<File>> {
    HTML::of(File::from("index.html"))
}

#[get("/error")]
pub fn error_page() -> Result<HTML<&'static str>> {
    Error::new(500, "Custom user error response")
}
