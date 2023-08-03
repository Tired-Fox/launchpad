mod file;
mod html;
mod json;
mod redirect;

use bytes::Bytes;
use http_body_util::Full;

pub use file::File;
pub use html::HTML;
use hyper::{Method, Uri};
pub use json::{Raw, JSON};
pub use redirect::Redirect;

pub type Result<T> = std::result::Result<T, (u16, String)>;

pub trait IntoString {
    fn into_string(self) -> String;
}

impl IntoString for String {
    fn into_string(self) -> String {
        self
    }
}
impl IntoString for &str {
    fn into_string(self) -> String {
        self.to_string()
    }
}

pub trait ToResponse {
    fn to_response(
        self,
        method: &Method,
        uri: &Uri,
        body: String,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>>;
}

pub trait ToErrorResponse {
    fn to_error_response(
        self,
        code: u16,
        reason: String,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>>;
}

impl<T: ToResponse> ToResponse for (u16, T) {
    fn to_response(
        self,
        method: &Method,
        uri: &Uri,
        body: String,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
        let code = self.0;
        self.1.to_response(method, uri, body).map(|result| {
            let mut response = hyper::Response::builder()
                .status(code)
                .body(result.body().clone())
                .unwrap();

            // Copy over all headers
            response.headers_mut().extend(result.headers().clone());

            response
        })
    }
}

impl<T: ToResponse> ToResponse for Result<T> {
    fn to_response(
        self,
        method: &Method,
        uri: &Uri,
        body: String,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
        match self {
            Ok(response) => response.to_response(method, uri, body),
            Err(error) => Err(error),
        }
    }
}

impl ToResponse for String {
    fn to_response(
        self,
        _method: &Method,
        _uri: &Uri,
        _body: String,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
        Ok(hyper::Response::builder()
            .status(200)
            .header("Content-Type", "text/plain")
            .body(Full::new(Bytes::from(self)))
            .unwrap())
    }
}

impl ToErrorResponse for String {
    fn to_error_response(
        self,
        code: u16,
        reason: String,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
        Ok(hyper::Response::builder()
            .status(code)
            .header("Content-Type", "text/plain")
            .header("Wayfinder-Reason", reason)
            .body(Full::new(Bytes::from(self)))
            .unwrap())
    }
}

impl ToResponse for &str {
    fn to_response(
        self,
        _method: &Method,
        _uri: &Uri,
        _body: String,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
        Ok(hyper::Response::builder()
            .status(200)
            .header("Content-Type", "text/plain")
            .body(Full::new(Bytes::from(self.to_string())))
            .unwrap())
    }
}

impl ToErrorResponse for &str {
    fn to_error_response(
        self,
        code: u16,
        reason: String,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
        Ok(hyper::Response::builder()
            .status(code)
            .header("Content-Type", "text/plain")
            .header("Wayfinder-Reason", reason)
            .body(Full::new(Bytes::from(self.to_string())))
            .unwrap())
    }
}
