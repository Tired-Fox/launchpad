use std::fmt::Display;

use bytes::Bytes;
use http_body_util::Full;

use super::{Result, ToErrorResponse, ToResponse};

pub struct Redirect<const CODE: u16 = 302>(pub String);

impl<const CODE: u16> Redirect<CODE> {
    pub fn to<T: Display>(value: T) -> Self {
        Redirect(value.to_string())
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
                .header("Wayfinder-Reason", reason)
                .body(Full::new(Bytes::new()))
                .unwrap())
        } else {
            Ok(hyper::Response::builder()
                .status(code)
                .header("Content-Type", "text/html")
                .header("Wayfinder-Reason", reason)
                .header("Location", self.0.to_string())
                .body(Full::new(Bytes::new()))
                .unwrap())
        }
    }
}

impl<const CODE: u16> ToResponse for Redirect<CODE> {
    fn to_response(self) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
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
