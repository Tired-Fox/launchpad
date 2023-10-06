use std::{convert::Infallible, fmt::Display, future::Future, pin::Pin};

use http_body_util::{Empty, Full};
use hyper::body::{Body, Bytes};
use serde::Deserialize;
use serde_json::Value;

use crate::error::Error;

/// Parse the body into repspective types.
///
/// The only required method to implement is `text` as all other types
/// are parsed from the result of that type
pub trait ParseBody<'r> {
    fn json<O>(self) -> Pin<Box<dyn Future<Output = Result<O, Error>> + Send>>
    where
        O: Deserialize<'r>,
        Self: Sized + 'static + Send,
    {
        Box::pin(async move {
            serde_json::from_str(Box::leak(self.text().await?.into_boxed_str()))
                .map_err(Error::from)
        })
    }

    fn text(self) -> Pin<Box<dyn Future<Output = Result<String, Error>> + Send>>;

    fn raw(self) -> Pin<Box<dyn Future<Output = Vec<u8>> + Send>>;

    fn primitive<O>(self) -> Pin<Box<dyn Future<Output = Result<O, Error>> + Send>>
    where
        O: Deserialize<'r>,
        Self: Sized + 'static + Send,
    {
        Box::pin(async move {
            serde_plain::from_str::<O>(Box::leak(self.text().await?.into_boxed_str()))
                .map_err(Error::from)
        })
    }
}

pub trait IntoBody<T>
where
    T: Body<Data = Bytes, Error = Infallible>,
{
    fn into_body(self) -> T;
}

impl IntoBody<Full<Bytes>> for &str {
    fn into_body(self) -> Full<Bytes> {
        Full::new(Bytes::from(self.to_string()))
    }
}

impl IntoBody<Full<Bytes>> for String {
    fn into_body(self) -> Full<Bytes> {
        Full::new(Bytes::from(self))
    }
}

impl IntoBody<Empty<Bytes>> for () {
    fn into_body(self) -> Empty<Bytes> {
        Empty::new()
    }
}

impl IntoBody<Full<Bytes>> for Value {
    fn into_body(self) -> Full<Bytes> {
        Full::new(Bytes::from(self.to_string()))
    }
}
