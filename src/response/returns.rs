use std::fmt::Display;

use http_body_util::Full;
use hyper::body::Bytes;

use crate::body::IntoBody;
use crate::error::Error;

#[cfg(feature = "macros")]
pub mod html {
    pub use crate::_html_new as new;
    pub use html_to_string_macro::html as string;

    #[macro_export]
    macro_rules! _html_new {
        ($($html: tt)*) => {
            $crate::response::HTML(
                $crate::response::html::string! {
                    $($html)*
                }
            )
        };
    }
}

#[cfg(feature = "macros")]
pub mod json {
    pub use crate::_json_new as new;
    pub use serde_json::json as object;

    #[macro_export]
    macro_rules! _json_new {
        ($($json: tt)*) => {
            $crate::response::JSON(
                $crate::response::json::object!($($json)*)
            )
        };
    }
}

use super::IntoResponse;

/// Light wrapper that sets `Content-Type` header to `text/html`
pub struct HTML<T>(pub T)
where
    T: IntoBody<Full<Bytes>>;

impl<T> From<T> for HTML<T>
where
    T: IntoBody<Full<Bytes>>,
{
    fn from(value: T) -> Self {
        HTML(value)
    }
}

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

impl<T> From<T> for JSON<T>
where
    T: IntoBody<Full<Bytes>>,
{
    fn from(value: T) -> Self {
        JSON(value)
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
