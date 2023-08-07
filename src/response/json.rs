use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Uri};
use serde::{Deserialize, Serialize};

use crate::errors::default_error_page;

use super::{File, Result, ToErrorResponse, ToResponse};

pub type Raw = serde_json::Value;

pub struct JSON<T: Serialize>(pub T);

impl<T: Deserialize<'static> + Serialize> JSON<T> {
    pub fn from_str(value: String) -> Result<Self> {
        match serde_json::from_str::<T>(Box::leak(value.into_boxed_str())) {
            Ok(obj) => Ok(JSON(obj)),
            _ => Err((500, "Failed to parse json from string".to_string())),
        }
    }

    pub fn from_file<U: Into<String> + Clone>(value: File<U>) -> Result<Self> {
        let path = Into::<String>::into(value.0.clone());
        match serde_json::from_str::<T>(Box::leak(Into::<String>::into(value).into_boxed_str())) {
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
