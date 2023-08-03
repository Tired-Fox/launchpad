use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Uri};

use super::{IntoString, Result, ToErrorResponse, ToResponse};

pub struct HTML<T: IntoString>(pub T);

impl<T: IntoString> ToResponse for HTML<T> {
    fn to_response(
        self,
        _method: &Method,
        _uri: &Uri,
        _body: String,
    ) -> Result<hyper::Response<Full<Bytes>>> {
        Ok(hyper::Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Full::new(Bytes::from(self.0.into_string())))
            .unwrap())
    }
}

impl<T: IntoString> ToErrorResponse for HTML<T> {
    fn to_error_response(
        self,
        code: u16,
        reason: String,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
        Ok(hyper::Response::builder()
            .status(code)
            .header("Content-Type", "text/html")
            .header("Wayfinder-Reason", reason)
            .body(Full::new(Bytes::from(self.0.into_string())))
            .unwrap())
    }
}
