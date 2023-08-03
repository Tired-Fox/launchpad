use std::{ffi::OsStr, fmt::Display, fs, path::Path};

use bytes::Bytes;
use http_body_util::Full;

use super::{IntoString, Result, ToErrorResponse, ToResponse};

pub struct File<T: Display>(pub T);

impl<T: Display> IntoString for File<T> {
    fn into_string(self) -> String {
        println!("Exists: {}", Path::new(&self.0.to_string()).exists());
        match fs::read_to_string(self.0.to_string()) {
            Ok(text) => text,
            _ => String::new(),
        }
    }
}

impl<T: Display> ToResponse for File<T> {
    fn to_response(self) -> Result<hyper::Response<Full<Bytes>>> {
        let ct = match Path::new(&self.0.to_string())
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
            .body(Full::new(Bytes::from(self.into_string())))
            .unwrap())
    }
}

impl<T: Display> ToErrorResponse for File<T> {
    fn to_error_response(self, code: u16, reason: String) -> Result<hyper::Response<Full<Bytes>>> {
        let ct = match Path::new(&self.0.to_string())
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
            .body(Full::new(Bytes::from(self.into_string())))
            .unwrap())
    }
}
