use std::fmt::Display;

use http_body_util::BodyExt;
use hyper::body::Incoming;
use serde::Deserialize;

pub type HttpRequest = hyper::Request<hyper::body::Incoming>;

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
            serde_json::error::Category::Data | serde_json::error::Category::Syntax => Category::Parse,
            _ => Category::General,
        }
    }
}

#[derive(Debug)]
pub struct RequestError {
    pub category: Category,
    pub message: String,
}

impl Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Error::{:?}] {}", self.category, self.message)
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(value: serde_json::Error) -> Self {
        RequestError {
            category: value.classify().into(),
            message: value.to_string(),
        }
    }
}
impl From<serde_plain::Error> for RequestError {
    fn from(value: serde_plain::Error) -> Self {
        use serde_plain::Error;
        let (category, message) = match value {
            Error::ImpossibleSerialization(message) | Error::ImpossibleDeserialization(message) => {
                (Category::Impossible, message.to_string())
            }
            Error::Parse(_, b) => (Category::Parse, b),
            Error::Message(message) => (Category::General, message),
        };
        RequestError {
            category,
            message: message.to_string(),
        }
    }
}
impl From<serde_qs::Error> for RequestError {
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
        RequestError {
            category,
            message: message.to_string(),
        }
    }
}

impl RequestError {
    fn new(category: Category, message: String) -> Self {
        RequestError { category, message }
    }
}

pub struct Request(HttpRequest);

impl From<HttpRequest> for Request {
    fn from(value: HttpRequest) -> Self {
        Request(value)
    }
}

impl From<Request> for HttpRequest {
    fn from(value: Request) -> Self {
        value.0
    }
}

impl<'r> Request {
    pub fn new(req: hyper::Request<Incoming>) -> Self {
        Request(req)
    }

    pub async fn json<T: Deserialize<'r>>(self) -> Result<T, RequestError> {
        serde_json::from_str(Box::leak(
            String::from_utf8(self.0.collect().await.unwrap().to_bytes().to_vec())
                .map_err(|e| RequestError::new(Category::Io, e.to_string()))?
                .into_boxed_str(),
        ))
        .map_err(|e| RequestError::from(e))
    }

    pub async fn text(self) -> Result<String, RequestError> {
        serde_plain::from_str::<String>(Box::leak(
            String::from_utf8(self.0.collect().await.unwrap().to_bytes().to_vec())
                .map_err(|e| RequestError::new(Category::Io, e.to_string()))?
                .into_boxed_str(),
        ))
        .map_err(|e| RequestError::from(e))
    }

    pub async fn body<T: Deserialize<'r>>(self) -> Result<T, RequestError> {
        serde_plain::from_str::<T>(Box::leak(
            String::from_utf8(self.0.collect().await.unwrap().to_bytes().to_vec())
                .map_err(|e| RequestError::new(Category::Io, e.to_string()))?
                .into_boxed_str(),
        ))
        .map_err(|e| RequestError::from(e))
    }

    pub fn query<T: Deserialize<'r>>(&self) -> Result<T, RequestError> {
        match self.0.uri().query() {
            Some(query) => serde_qs::from_str::<T>(Box::leak(String::from(query).into_boxed_str()))
                .map_err(|e| RequestError::from(e)),
            None => {
                return Err(RequestError::new(
                    Category::Io,
                    "No query available to parse".to_string(),
                ))
            }
        }
    }
}
