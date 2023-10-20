use std::convert::Infallible;
use std::fmt::Display;
use std::future::Future;
use std::sync::Arc;
use std::{collections::HashMap, pin::Pin};

use http_body_util::BodyExt;
use hyper::{
    body::{Bytes, Incoming},
    http::{request::Parts, HeaderValue},
    HeaderMap, Request as HttpRequest,
};
use serde::Deserialize;

use crate::body::{IntoBody, ParseBody};
use crate::error::Error;
use crate::server::State;

pub use hyper::{Method, Uri, Version};

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

impl<'r> ParseBody<'r> for hyper::Request<Incoming> {
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

impl<'r> ParseBody<'r> for Request {
    fn text(
        self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Error>> + Send>> {
        Box::pin(async move {
            String::from_utf8(self.0.collect().await.unwrap().to_bytes().to_vec())
                .map_err(Error::from)
        })
    }

    fn raw(self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<u8>> + Send>> {
        Box::pin(async move { self.0.collect().await.unwrap().to_bytes().to_vec() })
    }
}

/// Represents the different parts of a requests head properites.
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
impl<'r> ParseBody<'r> for Body {
    fn text(
        self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Error>> + Send>> {
        Box::pin(async move {
            String::from_utf8(self.0.collect().await.unwrap().to_bytes().to_vec())
                .map_err(Error::from)
        })
    }

    fn raw(self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<u8>> + Send>> {
        Box::pin(async move { self.0.collect().await.unwrap().to_bytes().to_vec() })
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

pub trait FromRequest
where
    Self: Send + Sized,
{
    fn from_request(request: &hyper::Request<Incoming>, state: Arc<State>) -> Result<Self, Error>;
}

impl<T> FromRequest for Option<T>
where
    T: FromRequest,
{
    fn from_request(request: &hyper::Request<Incoming>, state: Arc<State>) -> Result<Self, Error> {
        Ok(T::from_request(request, state).ok())
    }
}

impl FromRequest for Version {
    fn from_request(request: &hyper::Request<Incoming>, _state: Arc<State>) -> Result<Self, Error> {
        Ok(request.version())
    }
}

impl FromRequest for Head {
    fn from_request(request: &hyper::Request<Incoming>, _state: Arc<State>) -> Result<Self, Error> {
        Ok(Head::new(request))
    }
}

impl FromRequest for Method {
    fn from_request(request: &hyper::Request<Incoming>, _state: Arc<State>) -> Result<Self, Error> {
        Ok(request.method().clone())
    }
}

impl FromRequest for HashMap<String, String> {
    fn from_request(request: &hyper::Request<Incoming>, _state: Arc<State>) -> Result<Self, Error> {
        Ok(request
            .headers()
            .iter()
            .map(|(hn, hv)| (hn.to_string(), hv.to_str().unwrap().to_string()))
            .collect())
    }
}

impl FromRequest for Headers {
    fn from_request(request: &hyper::Request<Incoming>, _state: Arc<State>) -> Result<Self, Error> {
        Ok(request.headers().clone())
    }
}

impl FromRequest for Uri {
    fn from_request(request: &hyper::Request<Incoming>, _state: Arc<State>) -> Result<Self, Error> {
        Ok(request.uri().clone())
    }
}

pub trait FromRequestBody
where
    Self: Sized + Send,
{
    fn from_request_body(
        request: hyper::Request<Incoming>,
        state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Error>> + Send>>;
}

impl<T: FromRequestBody> FromRequestBody for Option<T> {
    fn from_request_body(
        request: hyper::Request<Incoming>,
        state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Error>> + Send>> {
        Box::pin(async { Ok(T::from_request_body(request, state).await.ok()) })
    }
}

impl FromRequestBody for Body {
    fn from_request_body(
        request: hyper::Request<Incoming>,
        _state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Error>> + Send>> {
        Box::pin(async { Ok(Body(request.into_body())) })
    }
}

impl FromRequestBody for Request {
    fn from_request_body(
        request: hyper::Request<Incoming>,
        _state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Error>> + Send>> {
        Box::pin(async { Ok(Request::from(request)) })
    }
}

impl FromRequestBody for String {
    fn from_request_body(
        request: hyper::Request<Incoming>,
        _state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Error>> + Send>> {
        Box::pin(Request::from(request).text())
    }
}

impl FromRequestBody for Version {
    fn from_request_body(
        request: hyper::Request<Incoming>,
        _state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Error>> + Send>> {
        Box::pin(async move { Ok(request.version()) })
    }
}

impl FromRequestBody for Head {
    fn from_request_body(
        request: hyper::Request<Incoming>,
        _state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Error>> + Send>> {
        Box::pin(async move { Ok(Head::new(&request)) })
    }
}

impl FromRequestBody for Method {
    fn from_request_body(
        request: hyper::Request<Incoming>,
        _state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Error>> + Send>> {
        Box::pin(async move { Ok(request.method().clone()) })
    }
}

impl FromRequestBody for HashMap<String, String> {
    fn from_request_body(
        request: hyper::Request<Incoming>,
        _state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Error>> + Send>> {
        Box::pin(async move {
            Ok(request
                .headers()
                .iter()
                .map(|(hn, hv)| (hn.to_string(), hv.to_str().unwrap().to_string()))
                .collect())
        })
    }
}

impl FromRequestBody for Headers {
    fn from_request_body(
        request: hyper::Request<Incoming>,
        _state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Error>> + Send>> {
        Box::pin(async move { Ok(request.headers().clone()) })
    }
}

impl FromRequestBody for Uri {
    fn from_request_body(
        request: hyper::Request<Incoming>,
        _state: Arc<State>,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Error>> + Send>> {
        Box::pin(async move { Ok(request.uri().clone()) })
    }
}
