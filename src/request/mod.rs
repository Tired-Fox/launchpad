use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Display;

use async_trait::async_trait;
use http_body_util::BodyExt;
use hyper::{
    body::{Bytes, Incoming},
    http::{request::Parts, HeaderValue},
    HeaderMap, Request as HttpRequest,
};
use serde::Deserialize;

use crate::body::{IntoBody, ParseBody};
use crate::error::Error;

pub use hyper::{Method, Uri, Version};

mod from_request;
mod from_request_parts;
pub use from_request::FromRequest;
pub use from_request_parts::FromRequestParts;

pub type Headers = HeaderMap<HeaderValue>;

/// Request builder
pub struct Builder {
    uri: String,
    headers: HashMap<String, String>,
    method: String,
    version: Version,
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            uri: String::new(),
            headers: HashMap::new(),
            method: String::from("GET"),
            version: Version::HTTP_10,
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Builder::default()
    }

    /// Set the request uri.
    pub fn uri<T>(mut self, uri: T) -> Self
    where
        T: ToString,
    {
        self.uri = uri.to_string().replace(" ", "%20");
        self
    }

    /// Add a request header.
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: ToString,
        V: Display,
    {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Set the reqeust method.
    pub fn method<M>(mut self, method: M) -> Self
    where
        M: ToString,
    {
        self.method = method.to_string();
        self
    }

    /// Set the request http version.
    pub fn version(mut self, version: f32) -> Self {
        if version == 0.9 {
            self.version = Version::HTTP_09;
        } else if version == 1.0 {
            self.version = Version::HTTP_10;
        } else if version == 1.1 {
            self.version = Version::HTTP_11;
        } else if version == 2.0 {
            self.version = Version::HTTP_2;
        } else if version == 3.0 {
            self.version = Version::HTTP_3;
        }
        self
    }

    /// Set the requests body and return the Request
    pub fn body<B, T>(self, body: T) -> HttpRequest<B>
    where
        B: hyper::body::Body<Data = Bytes, Error = Infallible>,
        T: IntoBody<B>,
    {
        let mut builder = HttpRequest::builder()
            .uri(self.uri)
            .method(self.method.as_str())
            .version(self.version);

        for (key, value) in self.headers.iter() {
            builder = builder.header(key, value);
        }

        builder.body(body.into_body()).unwrap()
    }
}

/// Wrapper around a hyper::Request<Incoming> that has helpers
/// for accessing and converting the data.
///
/// This object also allows for sending a request as a client with the `SendRequest` trait.
pub struct Request(HttpRequest<Incoming>);

impl From<HttpRequest<Incoming>> for Request {
    fn from(value: HttpRequest<Incoming>) -> Self {
        Request(value)
    }
}

impl From<Request> for HttpRequest<Incoming> {
    fn from(value: Request) -> Self {
        value.0
    }
}

#[async_trait]
impl<'r> ParseBody<'r> for hyper::Request<Incoming> {
    async fn text(self) -> Result<String, Error> {
        String::from_utf8(self.collect().await.unwrap().to_bytes().to_vec()).map_err(Error::from)
    }

    async fn raw(self) -> Vec<u8> {
        self.collect().await.unwrap().to_bytes().to_vec()
    }
}

#[async_trait]
impl<'r> ParseBody<'r> for Request {
    async fn text(self) -> Result<String, Error> {
        String::from_utf8(self.0.collect().await.unwrap().to_bytes().to_vec()).map_err(Error::from)
    }

    async fn raw(self) -> Vec<u8> {
        self.0.collect().await.unwrap().to_bytes().to_vec()
    }
}

/// Represents the different parts of a requests head properites.
#[derive(Clone)]
pub struct Head {
    pub method: Method,
    pub version: Version,
    pub headers: HeaderMap<HeaderValue>,
    pub uri: Uri,
}

impl From<Parts> for Head {
    fn from(value: Parts) -> Self {
        Head {
            method: value.method,
            version: value.version,
            headers: value.headers,
            uri: value.uri,
        }
    }
}

impl Head {
    pub fn new(request: &hyper::Request<Incoming>) -> Self {
        Head {
            method: request.method().clone(),
            version: request.version(),
            headers: request.headers().clone(),
            uri: request.uri().clone(),
        }
    }
}

/// Wrapper around a request `hyper::body::Incoming` body.
///
/// This wrapper has utility methods for converting the body to another data type.
pub struct Body(Incoming);
#[async_trait]
impl<'r> ParseBody<'r> for Body {
    async fn text(self) -> Result<String, Error> {
        String::from_utf8(self.0.collect().await.unwrap().to_bytes().to_vec()).map_err(Error::from)
    }

    async fn raw(self) -> Vec<u8> {
        self.0.collect().await.unwrap().to_bytes().to_vec()
    }
}

impl<'r> Request {
    /// A new wrapper around a hyper::Request.
    pub fn new(req: HttpRequest<Incoming>) -> Self {
        Request::from(req)
    }

    /// Build a request.
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Split the request into it's `Head` and `Body`.
    pub fn parts(self) -> (Head, Body) {
        let (head, body) = self.0.into_parts();
        (Head::from(head), Body(body))
    }

    pub fn version(&self) -> hyper::Version {
        self.0.version()
    }

    pub fn headers(&self) -> &hyper::HeaderMap<hyper::http::HeaderValue> {
        self.0.headers()
    }

    pub fn uri(&self) -> &hyper::Uri {
        self.0.uri()
    }

    pub fn method(&self) -> &hyper::Method {
        self.0.method()
    }

    /// Get the uri's query as another data type.
    pub fn query<T: Deserialize<'r>>(&self) -> Result<T, String> {
        match self.0.uri().query() {
            Some(query) => serde_qs::from_str::<T>(Box::leak(String::from(query).into_boxed_str()))
                .map_err(|e| e.to_string()),
            None => Err("No query available to parse".to_string()),
        }
    }
}

#[derive(Clone)]
pub struct State<T>(pub T);
pub trait FromStateRef<T: Clone>
where
    Self: Sized,
{
    fn from_state_ref(state: &T) -> State<Self>;
}

impl<T: Clone> FromStateRef<T> for T {
    fn from_state_ref(state: &T) -> State<Self> {
        State(state.clone())
    }
}
