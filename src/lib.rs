use std::fmt::Display;

pub mod server;
pub mod prelude;
pub mod template;
pub mod router;

pub use router::Router;
pub use server::Server;

// pub trait IntoBytes: Sized {
//     fn into_bytes(self) -> bytes::Bytes;
// }

// impl IntoBytes for &str {
//     fn into_bytes(self) -> bytes::Bytes {
//         bytes::Bytes::from(self)
//     }
// }
// impl IntoBytes for String {
//     fn into_bytes(self) -> bytes::Bytes {
//         bytes::Bytes::from(self)
//     }
// }
// impl IntoBytes for &[u8] {
//     fn into_bytes(self) -> bytes::Bytes {
//         bytes::Bytes::from(self)
//     }
// }
// impl IntoBytes for Vec<u8> {
//     fn into_bytes(self) -> bytes::Bytes {
//         bytes::Bytes::from(self)
//     }
// }

pub trait IntoParam<Result> {
    fn into_response(self) -> Result;
}

impl IntoParam<Option<String>> for Option<String> {
    fn into_response(self) -> Option<String> {
        self
    }
}

impl IntoParam<Option<String>> for String {
    fn into_response(self) -> Option<String> {
        Some(self)
    }
}

impl IntoParam<Option<String>> for &str {
    fn into_response(self) -> Option<String> {
        Some(self.to_string())
    }
}

pub enum Response {
    Success(bytes::Bytes),
    Error(u16)
}

impl Response {
    fn success<RES>(data: RES) -> Self 
    where RES: Display {
        Response::Success(bytes::Bytes::from(data.to_string()))
    }
}

impl From<u16> for Response {
    fn from(value: u16) -> Self {
        Response::Error(value)
    }
}

impl From<String> for Response {
    fn from(value: String) -> Self {
        Response::Success(bytes::Bytes::from(value))
    }
}

impl From<&str> for Response {
    fn from(value: &str) -> Self {
        Response::Success(bytes::Bytes::from(value.to_string()))
    }
}

pub type RouteCallback = fn(Option<String>) -> Response;
