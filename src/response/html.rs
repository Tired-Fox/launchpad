use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Uri};

use super::{Result, ToErrorResponse, ToResponse};

pub struct HTML<T: Into<String>>(pub T);

impl<T: Into<String>> ToResponse for HTML<T> {
    fn to_response(
        self,
        _method: &Method,
        _uri: &Uri,
        _body: String,
    ) -> Result<hyper::Response<Full<Bytes>>> {
        Ok(hyper::Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Full::new(Bytes::from(Into::<String>::into(self.0))))
            .unwrap())
    }
}

impl<T: Into<String>> ToErrorResponse for HTML<T> {
    fn to_error_response(
        self,
        code: u16,
        reason: String,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
        Ok(hyper::Response::builder()
            .status(code)
            .header("Content-Type", "text/html")
            .header("Wayfinder-Reason", reason)
            .body(Full::new(Bytes::from(Into::<String>::into(self.0))))
            .unwrap())
    }
}
