mod body;
mod query;
mod request_data;

pub use body::Body;
pub use query::Query;
pub use request_data::{RequestData, ToParam};

use bytes::Bytes;
use http_body_util::Full;
use std::fmt::Debug;

use crate::response::Result;

pub trait Endpoint: Sync + Send + Debug {
    fn methods(&self) -> Vec<hyper::Method>;
    fn path(&self) -> String;
    fn execute(
        &self,
        method: &hyper::Method,
        uri: &mut hyper::Uri,
        body: &mut Vec<u8>,
    ) -> Result<hyper::Response<Full<Bytes>>>;
}

pub trait Catch: Send + Sync + Debug {
    fn execute(
        &self,
        code: u16,
        message: String,
        reason: String,
    ) -> Result<hyper::Response<Full<Bytes>>>;
    fn code(&self) -> u16;
}
