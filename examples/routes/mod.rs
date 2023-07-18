extern crate launchpad;
use launchpad::{prelude::*, response::{HTML, File}, Result};

pub mod api;

#[get("/")]
pub fn index() -> Result<HTML<File>> {
    HTML::of(File::from("index.html"))
}
