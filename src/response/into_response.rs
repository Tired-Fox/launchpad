use http_body_util::{Empty, Full};
use hyper::{body::Bytes, Response as HttpResponse};
use tela_html::Element;

use crate::{body::IntoBody, prelude::Error};

/// Convert the current object into a `hyper::Response<http_body_utils::Full<hyper::body::Bytes>>`.
pub trait IntoResponse {
    fn into_response(self) -> HttpResponse<Full<Bytes>>;
}

impl IntoResponse for () {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        match hyper::Response::builder().body(Full::new(Bytes::new())) {
            Ok(v) => v,
            Err(e) => Error::from(e).into_response(),
        }
    }
}

impl IntoResponse for &str {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        match hyper::Response::builder()
            .status(200)
            .header("Content-Type", "text/plain")
            .body(Full::new(Bytes::from(self.to_string())))
        {
            Ok(v) => v,
            Err(e) => Error::from(e).into_response(),
        }
    }
}

impl IntoResponse for String {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        match hyper::Response::builder()
            .status(200)
            .header("Content-Type", "text/plain")
            .body(Full::new(Bytes::from(self)))
        {
            Ok(v) => v,
            Err(e) => Error::from(e).into_response(),
        }
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        match hyper::Response::builder()
            .status(200)
            .header("Content-Type", "application/octet-stream")
            .body(Full::new(Bytes::from(self)))
        {
            Ok(v) => v,
            Err(e) => Error::from(e).into_response(),
        }
    }
}

impl IntoResponse for &[u8] {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        match hyper::Response::builder()
            .status(200)
            .header("Content-Type", "application/octet-stream")
            .body(Full::new(Bytes::from(self.to_vec())))
        {
            Ok(v) => v,
            Err(e) => Error::from(e).into_response(),
        }
    }
}

impl<const SIZE: usize> IntoResponse for [u8; SIZE] {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        match hyper::Response::builder()
            .status(200)
            .header("Content-Type", "application/octet-stream")
            .body(Full::new(Bytes::from(self.to_vec())))
        {
            Ok(v) => v,
            Err(e) => Error::from(e).into_response(),
        }
    }
}

impl<T> IntoResponse for Result<T, Error>
where
    T: IntoResponse,
{
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        match self {
            Ok(v) => v.into_response(),
            Err(e) => e.into_response(),
        }
    }
}

impl IntoResponse for Full<Bytes> {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        hyper::Response::builder().status(200).body(self).unwrap()
    }
}

impl IntoResponse for Empty<Bytes> {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        hyper::Response::builder()
            .status(200)
            .body(Full::new(Bytes::new()))
            .unwrap()
    }
}

impl IntoResponse for Element {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        hyper::Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(self.into_body())
            .unwrap()
    }
}

impl IntoResponse for serde_json::Value {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        hyper::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(self.into_body())
            .unwrap()
    }
}
