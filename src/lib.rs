pub mod endpoint;
pub mod prelude;

pub mod router;
pub mod server;
pub mod state;

pub use router::Router;
pub use server::Server;
pub use state::State;


use bytes::Bytes;
use endpoint::Responder;

pub type Result<T> = std::result::Result<T, u16>;

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
