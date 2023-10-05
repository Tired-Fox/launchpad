use std::{convert::Infallible, fmt::Display, future::Future, pin::Pin};

use http_body_util::{Empty, Full};
use hyper::body::{Body, Bytes};
use serde::Deserialize;

#[derive(Debug)]
pub enum Category {
    Io,
    Impossible,
    Parse,
    General,
}

impl From<serde_json::error::Category> for Category {
    fn from(value: serde_json::error::Category) -> Self {
        match value {
            serde_json::error::Category::Io => Category::Io,
            serde_json::error::Category::Data | serde_json::error::Category::Syntax => {
                Category::Parse
            }
            _ => Category::General,
        }
    }
}

#[derive(Debug)]
pub struct BodyError {
    pub category: Category,
    pub message: String,
}

impl Display for BodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Error::{:?}] {}", self.category, self.message)
    }
}

impl From<serde_json::Error> for BodyError {
    fn from(value: serde_json::Error) -> Self {
        BodyError {
            category: value.classify().into(),
            message: value.to_string(),
        }
    }
}
impl From<serde_plain::Error> for BodyError {
    fn from(value: serde_plain::Error) -> Self {
        use serde_plain::Error;
        let (category, message) = match value {
            Error::ImpossibleSerialization(message) | Error::ImpossibleDeserialization(message) => {
                (Category::Impossible, message.to_string())
            }
            Error::Parse(_, b) => (Category::Parse, b),
            Error::Message(message) => (Category::General, message),
        };
        BodyError {
            category,
            message: message.to_string(),
        }
    }
}
impl From<serde_qs::Error> for BodyError {
    fn from(value: serde_qs::Error) -> Self {
        use serde_qs::Error;
        let (category, message) = match value {
            Error::FromUtf8(error) => (Category::Parse, error.to_string()),
            Error::Utf8(error) => (Category::Parse, error.to_string()),
            Error::ParseInt(error) => (Category::Parse, error.to_string()),
            Error::Custom(message) => (Category::General, message),
            Error::Parse(message, _) => (Category::Parse, message),
            Error::Unsupported => (
                Category::Impossible,
                "Query parsing not supported in this context".to_string(),
            ),
            Error::Io(error) => (Category::Io, error.to_string()),
        };
        BodyError {
            category,
            message: message.to_string(),
        }
    }
}

impl BodyError {
    pub fn new(category: Category, message: String) -> Self {
        BodyError { category, message }
    }
}

/// Parse the body into repspective types.
///
/// The only required method to implement is `text` as all other types
/// are parsed from the result of that type
pub trait ParseBody<'r> {
    fn json<O>(self) -> Pin<Box<dyn Future<Output = Result<O, BodyError>> + Send>>
    where
        O: Deserialize<'r>,
        Self: Sized + 'static + Send,
    {
        Box::pin(async move {
            serde_json::from_str(Box::leak(self.text().await?.into_boxed_str()))
                .map_err(|e| BodyError::from(e))
        })
    }

    fn text(self) -> Pin<Box<dyn Future<Output = Result<String, BodyError>> + Send>>;

    fn primitive<O>(self) -> Pin<Box<dyn Future<Output = Result<O, BodyError>> + Send>>
    where
        O: Deserialize<'r>,
        Self: Sized + 'static + Send,
    {
        Box::pin(async move {
            serde_plain::from_str::<O>(Box::leak(self.text().await?.into_boxed_str()))
                .map_err(|e| BodyError::from(e))
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
