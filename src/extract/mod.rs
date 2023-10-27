use std::fmt::{Debug, Display};
use std::sync::Arc;

use async_trait::async_trait;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::header::HeaderValue;
use hyper::{HeaderMap, Method, StatusCode, Uri, Version};
use serde::Deserialize;

use crate::body::ParseBody;
use crate::error::Error;
use crate::server::Parts;
use crate::Request;

mod from_request;
mod from_request_parts;

pub use from_request::{FromRequest, FromRequestOrParts};
pub use from_request_parts::FromRequestParts;

/// Extractor for x-www-form-urlencoded request Content-Type
pub struct Form<T>(pub T)
where
    T: Deserialize<'static>;

impl<T> Debug for Form<T>
where
    T: Deserialize<'static> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Form").field("content", &self.0).finish()
    }
}

impl<T> Display for Form<T>
where
    T: Deserialize<'static> + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> From<T> for Form<T>
where
    T: Deserialize<'static>,
{
    fn from(value: T) -> Self {
        Form(value)
    }
}

#[async_trait]
impl<T: Deserialize<'static> + Send, U: Send + Sync + 'static> FromRequest<U> for Form<T> {
    async fn from_request(
        request: hyper::Request<Incoming>,
        _: Arc<Parts<U>>,
    ) -> Result<Self, Error> {
        Request::from(request).form::<T>().await.map(|v| Form(v))
    }
}

/// Extractor for request query parameters
pub struct Query<T>(pub T)
where
    T: Deserialize<'static>;

impl<T> Debug for Query<T>
where
    T: Deserialize<'static> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Query").field("content", &self.0).finish()
    }
}

impl<T> Display for Query<T>
where
    T: Deserialize<'static> + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> From<T> for Query<T>
where
    T: Deserialize<'static>,
{
    fn from(value: T) -> Self {
        Query(value)
    }
}

impl<T: Deserialize<'static> + Send, U> FromRequestParts<U> for Query<T> {
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        _: Arc<Parts<U>>,
    ) -> Result<Self, Error> {
        let query = match request.uri().query() {
            Some(query) => query,
            None => {
                return Err(Error::from((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Uri does not contain a query",
                )))
            }
        };

        let static_query = Box::leak(query.to_string().into_boxed_str());
        match serde_qs::from_str::<T>(static_query) {
            Ok(value) => Ok(Query(value)),
            Err(err) => {
                use serde_qs::Error as qsError;
                match err {
                    qsError::Custom(_) => match serde_plain::from_str::<T>(static_query) {
                        Ok(value) => Ok(Query(value)),
                        _ => Err(Error::from(err)),
                    },
                    err => Err(Error::from(err)),
                }
            }
        }
    }
}

#[async_trait]
impl<T: Deserialize<'static> + Send, U: Send + Sync + 'static> FromRequest<U> for Query<T> {
    async fn from_request(
        request: hyper::Request<Incoming>,
        parts: Arc<Parts<U>>,
    ) -> Result<Self, Error> {
        Query::<T>::from_request_parts(&request, parts)
    }
}

/// Extractor for Json deserializable objects
pub struct Json<T>(pub T)
where
    T: Deserialize<'static>;

impl<T> From<T> for Json<T>
where
    T: Deserialize<'static>,
{
    fn from(value: T) -> Self {
        Json(value)
    }
}

#[async_trait]
impl<T: Deserialize<'static> + Send, U: Send + Sync + 'static> FromRequest<U> for Json<T> {
    async fn from_request(
        request: hyper::Request<Incoming>,
        _: Arc<Parts<U>>,
    ) -> Result<Self, Error> {
        Request::from(request).json::<T>().await.map(|v| Json(v))
    }
}

pub type Headers = HeaderMap<HeaderValue>;
/// Represents the different parts of a requests head properites.
#[derive(Clone)]
pub struct Head {
    pub method: Method,
    pub version: Version,
    pub headers: HeaderMap<HeaderValue>,
    pub uri: Uri,
}

impl From<hyper::http::request::Parts> for Head {
    fn from(value: hyper::http::request::Parts) -> Self {
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
pub struct Body(pub Incoming);
#[async_trait]
impl<'r> ParseBody<'r> for Body {
    async fn text(self) -> Result<String, Error> {
        String::from_utf8(self.0.collect().await.unwrap().to_bytes().to_vec()).map_err(Error::from)
    }

    async fn raw(self) -> Vec<u8> {
        self.0.collect().await.unwrap().to_bytes().to_vec()
    }
}
