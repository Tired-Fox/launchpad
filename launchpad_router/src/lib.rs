use bytes::Bytes;
use std::fmt::Display;

mod handler;
mod macros;
mod router;

pub mod endpoint;
pub mod request;
pub mod response;

pub use endpoint::Responder;
pub use endpoint::{Endpoint, ErrorCatch};
pub use handler::RouteHandler;
pub use router::{Catch, Command, Route, Router};

pub static ROOT: &'static str = "web";

#[derive(Debug)]
pub struct Error(u16, Option<String>);
impl From<u16> for Error {
    fn from(value: u16) -> Self {
        Error(value, None)
    }
}
impl<ToString: Display> From<(u16, ToString)> for Error {
    fn from(value: (u16, ToString)) -> Self {
        Error(value.0, Some(value.1.to_string()))
    }
}

impl Error {
    pub fn code(&self) -> &u16 {
        &self.0
    }

    pub fn message(&self) -> Option<&String> {
        self.1.as_ref()
    }

    pub fn new<ToString: Display>(code: u16, message: ToString) -> Self {
        Error(code, Some(message.to_string()))
    }

    pub fn of<T, ToString: Display>(code: u16, message: ToString) -> std::result::Result<T, Error> {
        Err(Error(code, Some(message.to_string())))
    }

    pub fn of_code<T>(code: u16) -> std::result::Result<T, Error> {
        Err(Error(code, None))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Response {
    // Content-Type, Body
    Success(String, bytes::Bytes),
    // Code, ?Message
    Error(u16, Option<String>),
}

impl<T: Responder> From<T> for Response {
    fn from(value: T) -> Self {
        match value.into_response() {
            Ok((content_type, data)) => Response::Success(content_type, data),
            Err(Error(code, message)) => Response::Error(code, message),
        }
    }
}

impl From<Error> for Response {
    fn from(value: Error) -> Self {
        Response::Error(value.0, value.1)
    }
}

impl<T: Display> Responder for T {
    fn into_response(self) -> std::result::Result<(String, bytes::Bytes), Error> {
        Ok(("text/plain".to_string(), Bytes::from(self.to_string())))
    }
}
