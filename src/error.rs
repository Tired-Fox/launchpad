use std::{fmt::Display, string::FromUtf8Error};

use http_body_util::Full;
use hyper::{
    body::Bytes,
    http::{status::InvalidStatusCode, HeaderValue},
    StatusCode,
};

use crate::{
    body::IntoBody,
    response::{IntoResponse, IntoStatusCode},
};

/// Tela's generic Error type
#[derive(Debug, Clone)]
pub struct Error {
    pub status: StatusCode,
    pub message: Option<String>,
    pub body: Option<Full<Bytes>>,
}

impl Error {
    pub fn new<T: Display, B: IntoBody<Full<Bytes>>>(
        status: StatusCode,
        message: Option<T>,
        body: Option<B>,
    ) -> Self {
        Error {
            status,
            message: message.map(|v| v.to_string()),
            body: body.map(|v| v.into_body()),
        }
    }

    pub fn status(status: StatusCode) -> Self {
        Error {
            status,
            message: None,
            body: None,
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
        Error::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            Some(value),
            None::<Full<Bytes>>,
        )
    }
}
impl From<serde_qs::Error> for Error {
    fn from(value: serde_qs::Error) -> Self {
        Error::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            Some(value),
            None::<Full<Bytes>>,
        )
    }
}
impl From<serde_plain::Error> for Error {
    fn from(value: serde_plain::Error) -> Self {
        Error::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            Some(value),
            None::<Full<Bytes>>,
        )
    }
}
impl From<hyper::http::Error> for Error {
    fn from(value: hyper::http::Error) -> Self {
        Error::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            Some(value),
            None::<Full<Bytes>>,
        )
    }
}
impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Error::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            Some(value),
            None::<Full<Bytes>>,
        )
    }
}
impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            Some(value),
            None::<Full<Bytes>>,
        )
    }
}
impl From<InvalidStatusCode> for Error {
    fn from(value: InvalidStatusCode) -> Self {
        Error::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            Some(value),
            None::<Full<Bytes>>,
        )
    }
}
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            Some(value),
            None::<Full<Bytes>>,
        )
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> hyper::Response<Full<Bytes>> {
        let body = match self.body {
            Some(body) => body,
            None => Full::new(Bytes::new()),
        };

        hyper::Response::builder()
            .status(self.status)
            .header(
                "Tela-Reason",
                match self.message {
                    None => HeaderValue::from_str("").unwrap(),
                    Some(message) => HeaderValue::from_str(&message).unwrap(),
                },
            )
            .body(body)
            .unwrap()
    }
}

impl<C: IntoStatusCode, M: Display, B: ToString> IntoResponse for (C, M, B) {
    fn into_response(self) -> hyper::Response<Full<Bytes>> {
        match hyper::Response::builder()
            .status(self.0.into_status_code())
            .header("Tela-Reason", self.1.to_string())
            .body(Full::new(Bytes::from(self.2.to_string())))
        {
            Ok(v) => v,
            Err(e) => Error::from(e).into_response(),
        }
    }
}

impl<S: IntoStatusCode, T: Display> IntoResponse for (S, T) {
    fn into_response(self) -> hyper::Response<Full<Bytes>> {
        match hyper::Response::builder()
            .status(self.0.into_status_code())
            .header("Tela-Reason", self.1.to_string())
            .body(Full::new(Bytes::new()))
        {
            Ok(v) => v,
            Err(e) => Error::from(e).into_response(),
        }
    }
}

impl From<u16> for Error {
    fn from(value: u16) -> Self {
        Error::new(
            value.into_status_code(),
            None::<String>,
            None::<Full<Bytes>>,
        )
    }
}

impl From<StatusCode> for Error {
    fn from(value: StatusCode) -> Self {
        Error::new(value, None::<String>, None::<Full<Bytes>>)
    }
}

impl<C: IntoStatusCode, M: Display> From<(C, M)> for Error {
    fn from(value: (C, M)) -> Self {
        Error::new(
            value.0.into_status_code(),
            Some(value.1.to_string()),
            None::<Full<Bytes>>,
        )
    }
}

impl<C: IntoStatusCode, M: Display, B: IntoBody<Full<Bytes>>> From<(C, M, B)> for Error {
    fn from(value: (C, M, B)) -> Self {
        Error {
            status: value.0.into_status_code(),
            message: Some(value.1.to_string()),
            body: Some(value.2.into_body()),
        }
    }
}
