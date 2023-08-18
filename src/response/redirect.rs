use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Uri};

use super::{Result, ToErrorResponse, ToResponse};

pub struct Redirect<const CODE: u16 = 302>(pub String);

impl<const CODE: u16> Redirect<CODE> {
    pub fn to<T: Into<String>>(value: T) -> Self {
        Redirect(Into::<String>::into(value))
    }
}

impl<const CODE: u16> ToErrorResponse for Redirect<CODE> {
    fn to_error_response(
        self,
        code: u16,
        reason: String,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
        if ![301, 302, 303, 307, 308].contains(&code) {
            Ok(hyper::Response::builder()
                .status(302)
                .header("Content-Type", "text/html")
                .header("Location", self.0.to_string())
                .header("Tela-Reason", reason)
                .body(Full::new(Bytes::new()))
                .unwrap())
        } else {
            Ok(hyper::Response::builder()
                .status(code)
                .header("Content-Type", "text/html")
                .header("Tela-Reason", reason)
                .header("Location", self.0.to_string())
                .body(Full::new(Bytes::new()))
                .unwrap())
        }
    }
}

impl<const CODE: u16> ToResponse for Redirect<CODE> {
    fn to_response(
        self,
        _method: &Method,
        _uri: &Uri,
        _body: String,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
        if ![301, 302, 303, 307, 308].contains(&CODE) {
            Ok(hyper::Response::builder()
                .status(302)
                .header("Content-Type", "text/html")
                .header("Location", self.0.to_string())
                .body(Full::new(Bytes::new()))
                .unwrap())
        } else {
            Ok(hyper::Response::builder()
                .status(CODE)
                .header("Content-Type", "text/html")
                .header("Location", self.0.to_string())
                .body(Full::new(Bytes::new()))
                .unwrap())
        }
    }
}
