use std::fmt::Display;

use http_body_util::Full;
use hyper::body::Bytes;

use crate::body::IntoBody;
use crate::error::Error;

#[cfg(feature = "macros")]
#[macro_export]
macro_rules! html {
    ($($html: tt)*) => {
       $crate::response::HTML(html_to_string_macro::html!{$($html)*})
    };
}
#[cfg(feature = "macros")]
pub use crate::html;

#[cfg(feature = "macros")]
#[macro_export]
macro_rules! json {
    ($($json: tt)*) => {
       $crate::response::JSON(serde_json::json!{$($json)*})
    };
}
#[cfg(feature = "macros")]
pub use crate::json;

use super::IntoResponse;

/// Light wrapper that sets `Content-Type` header to `text/html`
pub struct HTML<T>(pub T)
where
    T: IntoBody<Full<Bytes>>;

impl<T> Display for HTML<T>
where
    T: IntoBody<Full<Bytes>> + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> Default for HTML<T>
where
    T: IntoBody<Full<Bytes>> + Default,
{
    fn default() -> Self {
        HTML(T::default())
    }
}

/// Light wrapper that sets `Content-Type` header to `application/json`
pub struct JSON<T>(pub T)
where
    T: IntoBody<Full<Bytes>>;

impl<T> IntoResponse for HTML<T>
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

impl<T> IntoResponse for JSON<T>
where
    T: IntoBody<Full<Bytes>>,
{
    fn into_response(self) -> hyper::Response<Full<Bytes>> {
        match hyper::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(self.0.into_body())
        {
            Ok(v) => v,
            Err(e) => Error::from(e).into_response(),
        }
    }
}

impl<T> IntoBody<Full<Bytes>> for HTML<T>
where
    T: IntoBody<Full<Bytes>>,
{
    fn into_body(self) -> Full<Bytes> {
        self.0.into_body()
    }
}

impl<T> IntoBody<Full<Bytes>> for JSON<T>
where
    T: IntoBody<Full<Bytes>>,
{
    fn into_body(self) -> Full<Bytes> {
        self.0.into_body()
    }
}
