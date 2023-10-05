pub use html_to_string_macro::html;
use http_body_util::{BodyExt, Full};
use hyper::{
    body::{Bytes, Incoming},
    Response as HttpResponse, StatusCode, Version,
};
use std::collections::HashMap;
use std::fmt::Display;

use crate::body::{BodyError, Category, ParseBody};

pub type Body = Full<Bytes>;

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

    pub fn status<S>(mut self, status: S) -> Self
    where
        S: IntoStatusCode,
    {
        self.response.status = status.into_status_code();
        self
    }

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

    pub fn body<B>(mut self, body: B) -> Response
    where
        B: Into<Bytes>,
    {
        self.response.body = Full::new(body.into());
        self.response
    }
}

impl<'r> ParseBody<'r> for Response {
    fn text(
        self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, crate::body::BodyError>> + Send>,
    > {
        Box::pin(async move {
            String::from_utf8(self.body.collect().await.unwrap().to_bytes().to_vec())
                .map_err(|e| BodyError::new(Category::Io, e.to_string()))
        })
    }

    fn raw(
        self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, BodyError>> + Send>>
    {
        Box::pin(async move { Ok(self.body.collect().await.unwrap().to_bytes().to_vec()) })
    }
}

impl<'r> ParseBody<'r> for hyper::Response<Incoming> {
    fn text(
        self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, crate::body::BodyError>> + Send>,
    > {
        Box::pin(async move {
            String::from_utf8(self.collect().await.unwrap().to_bytes().to_vec())
                .map_err(|e| BodyError::new(Category::Io, e.to_string()))
        })
    }

    fn raw(
        self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, BodyError>> + Send>>
    {
        Box::pin(async move { Ok(self.collect().await.unwrap().to_bytes().to_vec()) })
    }
}

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
        StatusCode::from_u16(self).unwrap()
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

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }
}

pub trait IntoResponse {
    fn into_response(self) -> HttpResponse<Full<Bytes>>;
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

        builder.body(self.body).unwrap()
    }
}

impl IntoResponse for &str {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        hyper::Response::builder()
            .status(200)
            .body(Full::new(Bytes::from(self.to_string())))
            .unwrap()
    }
}

impl IntoResponse for String {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        hyper::Response::builder()
            .status(200)
            .body(Full::new(Bytes::from(self)))
            .unwrap()
    }
}
