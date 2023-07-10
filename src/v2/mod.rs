pub mod endpoint;
pub mod prelude;

pub mod router;
pub mod server;
pub mod state;

pub use router::Router;
pub use server::Server;

use std::fmt::Display;

use bytes::Bytes;
use endpoint::Responder;

pub enum Response {
    Success(bytes::Bytes),
    Error(u16),
}

impl<T: Responder> From<T> for Response {
    fn from(value: T) -> Self {
        Response::Success(value.into_response())
    }
}

impl From<u16> for Response {
    fn from(value: u16) -> Self {
        Response::Error(value)
    }
}

impl Response {
    fn success<RES>(data: RES) -> Self
    where
        RES: Display,
    {
        Response::Success(bytes::Bytes::from(data.to_string()))
    }
}

// Default Responder implmentation types
impl Responder for &str {
    fn into_response(self) -> bytes::Bytes {
        Bytes::from(self.to_string())
    }
}
impl Responder for String {
    fn into_response(self) -> bytes::Bytes {
        Bytes::from(self)
    }
}
impl Responder for &[u8] {
    fn into_response(self) -> bytes::Bytes {
        Bytes::from(self.to_vec())
    }
}
impl Responder for Vec<u8> {
    fn into_response(self) -> bytes::Bytes {
        Bytes::from(self)
    }
}
impl Responder for Bytes {
    fn into_response(self) -> bytes::Bytes {
        self
    }
}
