use std::{convert::Infallible, fmt::Display, string::FromUtf8Error};

use http_body_util::Full;
use hyper::{body::Bytes, http::status::InvalidStatusCode, StatusCode};

use crate::response::{IntoResponse, IntoStatusCode};

#[derive(Debug, Clone)]
pub struct Error {
    pub status: StatusCode,
    pub message: Option<String>,
}

impl Error {
    pub fn new<T: ToString>(status: StatusCode, message: Option<T>) -> Self {
        Error {
            status,
            message: message.map(|v| v.to_string()),
        }
    }

    pub fn status(status: StatusCode) -> Self {
        Error {
            status,
            message: None,
        }
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.status,
            match &self.message {
                Some(message) => format!("{}: ", message),
                None => String::new(),
            }
        )
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::new(StatusCode::INTERNAL_SERVER_ERROR, Some(value))
    }
}
impl From<serde_qs::Error> for Error {
    fn from(value: serde_qs::Error) -> Self {
        Error::new(StatusCode::INTERNAL_SERVER_ERROR, Some(value))
    }
}
impl From<serde_plain::Error> for Error {
    fn from(value: serde_plain::Error) -> Self {
        Error::new(StatusCode::INTERNAL_SERVER_ERROR, Some(value))
    }
}
impl From<hyper::http::Error> for Error {
    fn from(value: hyper::http::Error) -> Self {
        Error::new(StatusCode::INTERNAL_SERVER_ERROR, Some(value))
    }
}
impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Error::new(StatusCode::INTERNAL_SERVER_ERROR, Some(value))
    }
}
impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::new(StatusCode::INTERNAL_SERVER_ERROR, Some(value))
    }
}
impl From<InvalidStatusCode> for Error {
    fn from(value: InvalidStatusCode) -> Self {
        Error::new(StatusCode::INTERNAL_SERVER_ERROR, Some(value))
    }
}
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::new(StatusCode::INTERNAL_SERVER_ERROR, Some(value))
    }
}

impl From<u16> for Error {
    fn from(value: u16) -> Self {
        Error::new(value.into_status_code(), None::<String>)
    }
}

impl<T: ToString> From<(u16, T)> for Error {
    fn from(value: (u16, T)) -> Self {
        Error::new(value.0.into_status_code(), Some(value.1))
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> hyper::Response<Full<Bytes>> {
        hyper::Response::builder()
            .status(self.status)
            .header(
                "Tela-Reason",
                match self.message {
                    None => String::new(),
                    Some(message) => message,
                },
            )
            .body(Full::new(Bytes::new()))
            .unwrap()
    }
}

impl<T: ToString> IntoResponse for (u16, T) {
    fn into_response(self) -> hyper::Response<Full<Bytes>> {
        match hyper::Response::builder()
            .status(self.0)
            .header("Tela-Reason", self.1.to_string())
            .body(Full::new(Bytes::new()))
        {
            Ok(v) => v,
            Err(e) => Error::from(e).into_response(),
        }
    }
}
