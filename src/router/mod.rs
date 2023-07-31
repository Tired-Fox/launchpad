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

impl Responder for Error {
    fn into_response(self) -> std::result::Result<(String, bytes::Bytes), Error> {
        Err(self)
    }
}

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
    pub fn try_code<U>(code: u16) -> Result<U> {
        Err(Error::new(code, None::<String>))
    }

    pub fn try_new<U, T: Display>(code: u16, message: T) -> Result<U> {
        Err(Error::new(code, Some(message)))
    }

    pub fn new<T: Display>(code: u16, message: Option<T>) -> Self {
        Error(code, message.map(|m| m.to_string()))
    }

    pub fn code(&self) -> &u16 {
        &self.0
    }

    pub fn message(&self) -> Option<&String> {
        self.1.as_ref()
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

impl<U: Responder> From<Result<U>> for Response {
    fn from(value: Result<U>) -> Self {
        match value {
            Ok(data) => match data.into_response() {
                Ok((content_type, data)) => Response::Success(content_type, data),
                Err(Error(code, message)) => Response::Error(code, message),
            },
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
