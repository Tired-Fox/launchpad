mod body;
mod query;

pub use body::Body;
pub use query::Query;

use bytes::Bytes;
use http_body_util::Full;
use std::convert::Infallible;

pub trait Endpoint {
    fn methods(&self) -> Vec<hyper::Method>;
    fn path(&self) -> &'static str;
    fn execute(
        &self,
        uri: &mut hyper::Uri,
        body: &mut Vec<u8>,
    ) -> Result<hyper::Response<Full<Bytes>>, Infallible>;
}

pub trait Catch {
    fn execute(&self, code: u16, message: String, reason: String) -> String;
    fn code(&self) -> u16;
}
