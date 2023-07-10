use std::fmt::Debug;

pub use hyper::Method;

use super::Response;

// PERF: Move to own file
pub struct Context;

pub trait Responder {
    fn into_response(self) -> bytes::Bytes;
}

pub type Result<T> = std::result::Result<T, u16>;

pub trait Endpoint: Debug + Sync + Send {
    fn call(&self) -> Response;
    fn path(&self) -> String;
    fn methods(&self) -> Vec<Method>;
}
