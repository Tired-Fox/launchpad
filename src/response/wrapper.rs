use std::fmt::{Debug, Display};

use http_body_util::Full;
use hyper::body::Bytes;
use serde::Serialize;

use crate::body::IntoBody;
use crate::error::Error;
use crate::prelude::IntoResponse;

/// Represents the html in a request or response body.
pub struct Html<T>(pub T)
    where
        T: IntoBody<Full<Bytes>>;

impl<T> From<T> for Html<T>
    where
        T: IntoBody<Full<Bytes>>,
{
    fn from(value: T) -> Self {
        Html(value)
    }
}

impl<T> Debug for Html<T>
    where
        T: IntoBody<Full<Bytes>> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Html").field("content", &self.0).finish()
    }
}

impl<T> Display for Html<T>
    where
        T: IntoBody<Full<Bytes>> + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> Default for Html<T>
    where
        T: IntoBody<Full<Bytes>> + Default,
{
    fn default() -> Self {
        Html(T::default())
    }
}

impl<T> IntoResponse for Html<T>
    where
        T: IntoBody<Full<Bytes>>,
{
    fn into_response(self) -> hyper::Response<Full<Bytes>> {
        match hyper::Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(self.0.into_body())
        {
            Ok(v) => v,
            Err(e) => Error::from(e).into_response(),
        }
    }
}

impl<T> IntoBody<Full<Bytes>> for Html<T>
    where
        T: IntoBody<Full<Bytes>>,
{
    fn into_body(self) -> Full<Bytes> {
        self.0.into_body()
    }
}

/// Represents the json in the request or response body.
pub struct Json<T>(pub T)
    where
        T: Serialize;


impl From<serde_json::Value> for Json<String> {
    fn from(value: serde_json::Value) -> Self {
        Json(value.to_string())
    }
}


impl<T> IntoBody<Full<Bytes>> for Json<T>
    where
        T: Serialize,
{
    fn into_body(self) -> Full<Bytes> {
        match serde_json::to_string(&self.0) {
            Ok(value) => value.into_body(),
            Err(_) => Full::default(),
        }
    }
}

impl<T> From<T> for Json<T>
    where
        T: Serialize,
{
    fn from(value: T) -> Self {
        Json(value)
    }
}

impl<T> IntoResponse for Json<T>
    where
        T: Serialize,
{
    fn into_response(self) -> hyper::Response<Full<Bytes>> {
        match hyper::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(self.into_body())
        {
            Ok(v) => v,
            Err(e) => Error::from(e).into_response(),
        }
    }
}