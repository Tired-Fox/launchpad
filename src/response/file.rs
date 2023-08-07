use std::{ffi::OsStr, fs, path::Path};

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Uri};

use super::{Result, ToErrorResponse, ToResponse};

pub struct File<T: Into<String> + Clone>(pub T);

impl<T: Into<String> + Clone> Into<String> for File<T> {
    fn into(self) -> String {
        match fs::read_to_string(Into::<String>::into(self.0)) {
            Ok(text) => text,
            _ => String::new(),
        }
    }
}

impl<T: Into<String> + Clone> ToResponse for File<T> {
    fn to_response(
        self,
        _method: &Method,
        _uri: &Uri,
        _body: String,
    ) -> Result<hyper::Response<Full<Bytes>>> {
        let ct = match Path::new(&Into::<String>::into(self.0.clone()))
            .extension()
            .and_then(OsStr::to_str)
        {
            Some(extension) => match extension {
                "html" | "htm" => "text/html",
                "json" => "application/json",
                _ => "text/plain",
            },
            _ => "text/plain",
        };

        Ok(hyper::Response::builder()
            .status(200)
            .header("Content-Type", ct)
            .body(Full::new(Bytes::from(Into::<String>::into(self))))
            .unwrap())
    }
}

impl<T: Into<String> + Clone> ToErrorResponse for File<T> {
    fn to_error_response(self, code: u16, reason: String) -> Result<hyper::Response<Full<Bytes>>> {
        let ct = match Path::new(&Into::<String>::into(self.0.clone()))
            .extension()
            .and_then(OsStr::to_str)
        {
            Some(extension) => match extension {
                "html" | "htm" => "text/html",
                "json" => "application/json",
                _ => "text/plain",
            },
            _ => "text/plain",
        };
        Ok(hyper::Response::builder()
            .status(code)
            .header("Content-Type", ct)
            .header("Wayfinder-Reason", reason)
            .body(Full::new(Bytes::from(Into::<String>::into(self))))
            .unwrap())
    }
}
