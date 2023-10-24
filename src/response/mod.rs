use async_trait::async_trait;
use http_body_util::{BodyExt, Full};
pub use hyper::StatusCode;
use hyper::{
    body::{Bytes, Incoming},
    Response as HttpResponse, Version,
};
use std::collections::HashMap;
use std::fmt::Display;

use crate::{body::ParseBody, error::Error};

mod into_response;
pub use into_response::IntoResponse;

/// Response builder
#[derive(Clone)]
pub struct Builder {
    response: Response,
}
impl Builder {
    pub fn new() -> Self {
        Builder {
            response: Response::default(),
        }
    }

    /// Set the http response status
    pub fn status<S>(mut self, status: S) -> Self
    where
        S: IntoStatusCode,
    {
        self.response.status = status.into_status_code();
        self
    }

    /// Add a response header.
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Display,
        V: Display,
    {
        self.response
            .headers
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Set the response body and return a Response.
    pub fn body<B>(mut self, body: B) -> Response
    where
        B: Into<Bytes>,
    {
        self.response.body = Full::new(body.into());
        self.response
    }
}

#[async_trait]
impl<'r> ParseBody<'r> for Response {
    async fn text(self) -> Result<String, Error> {
        String::from_utf8(self.body.collect().await.unwrap().to_bytes().to_vec())
            .map_err(Error::from)
    }

    async fn raw(self) -> Vec<u8> {
        self.body.collect().await.unwrap().to_bytes().to_vec()
    }
}

#[async_trait]
impl<'r> ParseBody<'r> for hyper::Response<Incoming> {
    async fn text(self) -> Result<String, Error> {
        String::from_utf8(self.collect().await.unwrap().to_bytes().to_vec()).map_err(Error::from)
    }

    async fn raw(self) -> Vec<u8> {
        self.collect().await.unwrap().to_bytes().to_vec()
    }
}

/// A http response representation including status, headers, http version, and body.
///
/// This is a rough clone of hyper::Response
#[derive(Clone)]
pub struct Response {
    status: StatusCode,
    headers: HashMap<String, String>,
    body: Full<Bytes>,
    version: Version,
}

impl Default for Response {
    fn default() -> Self {
        Response {
            status: StatusCode::OK,
            headers: HashMap::new(),
            body: Full::new(Bytes::new()),
            version: Version::HTTP_10,
        }
    }
}

pub trait IntoStatusCode {
    fn into_status_code(self) -> StatusCode;
}
impl IntoStatusCode for StatusCode {
    fn into_status_code(self) -> StatusCode {
        self
    }
}
impl IntoStatusCode for u16 {
    fn into_status_code(self) -> StatusCode {
        StatusCode::from_u16(self).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl Response {
    pub fn builder() -> Builder {
        Builder::new()
    }
    pub fn new() -> Response {
        Response::default()
    }

    pub fn status(&self) -> &StatusCode {
        &self.status
    }

    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.status
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.headers
    }

    pub fn body(&self) -> &Full<Bytes> {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut Full<Bytes> {
        &mut self.body
    }
}

impl IntoResponse for Builder {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        self.clone().body(Bytes::new()).into_response()
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        let mut builder = hyper::Response::builder()
            .status(self.status)
            .version(self.version);

        for (key, value) in self.headers.iter() {
            builder = builder.header(key, value)
        }

        match builder.body(self.body) {
            Ok(v) => v,
            Err(e) => Error::from(e).into_response(),
        }
    }
}
