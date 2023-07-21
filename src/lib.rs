pub mod endpoint;
pub mod prelude;

pub mod request;
pub mod response;
pub mod router;
pub mod server;
mod handler;

pub mod support;

use std::fmt::Display;

pub use router::Router;
pub use server::Server;

use bytes::Bytes;
use endpoint::Responder;

static ROOT: &'static str = "web";

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
    pub fn new<T, ToString: Display>(
        code: u16,
        message: ToString,
    ) -> std::result::Result<T, Error> {
        Err(Error(code, Some(message.to_string())))
    }

    pub fn code<T>(code: u16) -> std::result::Result<T, Error> {
        Err(Error(code, None))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub enum Response {
    Success(String, bytes::Bytes),
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

impl From<u16> for Response {
    fn from(value: u16) -> Self {
        Response::Error(value, None)
    }
}

impl From<(u16, String)> for Response {
    fn from(value: (u16, String)) -> Self {
        Response::Error(value.0, Some(value.1))
    }
}

// Default Responder implmentation types
impl Responder for &str {
    fn into_response(self) -> std::result::Result<(String, bytes::Bytes), Error> {
        Ok(("text/plain".to_string(), Bytes::from(self.to_string())))
    }
}
impl Responder for String {
    fn into_response(self) -> std::result::Result<(String, bytes::Bytes), Error> {
        Ok(("text/plain".to_string(), Bytes::from(self)))
    }
}
impl Responder for &[u8] {
    fn into_response(self) -> std::result::Result<(String, bytes::Bytes), Error> {
        Ok(("text/plain".to_string(), Bytes::from(self.to_vec())))
    }
}
impl Responder for Vec<u8> {
    fn into_response(self) -> std::result::Result<(String, bytes::Bytes), Error> {
        Ok(("text/plain".to_string(), Bytes::from(self)))
    }
}
impl Responder for Bytes {
    fn into_response(self) -> std::result::Result<(String, bytes::Bytes), Error> {
        Ok(("text/plain".to_string(), self))
    }
}
