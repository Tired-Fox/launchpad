use http_body_util::{BodyExt, Empty, Full};
use hyper::{
    body::{Bytes, Incoming},
    Response as HttpResponse, StatusCode, Version,
};
use std::{collections::HashMap, fs, io::Read};
use std::{fmt::Display, path::PathBuf};
use tela_html::Element;

use crate::{
    body::{IntoBody, ParseBody},
    error::Error,
};

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
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Error>> + Send>> {
        Box::pin(async move {
            String::from_utf8(self.body.collect().await.unwrap().to_bytes().to_vec())
                .map_err(Error::from)
        })
    }

    fn raw(self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<u8>> + Send>> {
        Box::pin(async move { self.body.collect().await.unwrap().to_bytes().to_vec() })
    }
}

impl<'r> ParseBody<'r> for hyper::Response<Incoming> {
    fn text(
        self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Error>> + Send>> {
        Box::pin(async move {
            String::from_utf8(self.collect().await.unwrap().to_bytes().to_vec())
                .map_err(Error::from)
        })
    }

    fn raw(self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<u8>> + Send>> {
        Box::pin(async move { self.collect().await.unwrap().to_bytes().to_vec() })
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

impl IntoBody<Full<Bytes>> for PathBuf {
    fn into_body(self) -> Full<Bytes> {
        match fs::read(self) {
            Ok(file) => Full::new(Bytes::from(file)),
            Err(e) => {
                eprintln!("Error while serving file: {}", e);
                Full::default()
            }
        }
    }
}

impl IntoResponse for PathBuf {
    fn into_response(self) -> HttpResponse<Full<Bytes>> {
        let mime = mime_guess::from_path(&self).first_or_text_plain();
        hyper::Response::builder()
            .status(200)
            .header("Content-Type", mime.to_string())
            .body(self.into_body())
            .unwrap()
    }
}
