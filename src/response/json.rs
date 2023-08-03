use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Uri};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::errors::default_error_page;

use super::{File, IntoString, Result, ToErrorResponse, ToResponse};

pub struct Raw(pub String);
impl Serialize for Raw {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Need to convert string into Value enum first to allow for the string to be treated
        // as an object instead of a json string literal. The resulting object is then serialized
        // back into a string
        serde_json::from_str::<serde_json::Value>(self.0.as_str())
            .unwrap()
            .serialize(serializer)
    }
}

impl From<String> for Raw {
    fn from(value: String) -> Self {
        Raw(value)
    }
}

impl From<&str> for Raw {
    fn from(value: &str) -> Self {
        Raw(value.to_string())
    }
}

pub struct JSON<T: Serialize>(pub T);

impl<T: Deserialize<'static> + Serialize> JSON<T> {
    pub fn from_str(value: String) -> Result<Self> {
        match serde_json::from_str::<T>(Box::leak(value.into_boxed_str())) {
            Ok(obj) => Ok(JSON(obj)),
            _ => Err((500, "Failed to parse json from string".to_string())),
        }
    }

    pub fn from_file<U: Display>(value: File<U>) -> Result<Self> {
        let path = value.0.to_string();
        match serde_json::from_str::<T>(Box::leak(value.into_string().into_boxed_str())) {
            Ok(obj) => Ok(JSON(obj)),
            Err(err) => Err((
                500,
                format!("Failed to parse json from file {:?}: {}", path, err),
            )),
        }
    }
}

impl<T: serde::Serialize> ToResponse for JSON<T> {
    fn to_response(
        self,
        method: &Method,
        uri: &Uri,
        body: String,
    ) -> Result<hyper::Response<Full<Bytes>>> {
        match serde_json::to_string(&self.0) {
            Ok(result) => Ok(hyper::Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(result)))
                .unwrap()),
            Err(_) => Ok(default_error_page(
                &500,
                &"Failed to parse json in response".to_string(),
                method,
                uri,
                body,
            )),
        }
    }
}

impl<T: serde::Serialize> ToErrorResponse for JSON<T> {
    fn to_error_response(self, code: u16, reason: String) -> Result<hyper::Response<Full<Bytes>>> {
        match serde_json::to_string(&self.0) {
            Ok(result) => Ok(hyper::Response::builder()
                .status(code)
                .header("Content-Type", "application/json")
                .header("Wayfinder-Reason", reason)
                .body(Full::new(Bytes::from(result)))
                .unwrap()),
            Err(_) => Ok(hyper::Response::builder()
                .status(500)
                .header("Content-Type", "text/html")
                .header(
                    "Wayfinder-Reason",
                    format!("{}{}", reason, "; Failed to parse json response"),
                )
                .body(Full::new(Bytes::new()))
                .unwrap()),
        }
    }
}
