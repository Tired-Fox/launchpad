use super::{Error, Responder, Result, ROOT};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, fs, path::PathBuf};

pub trait ResType {}

pub struct JSON<T: Sized + Serialize>(pub T);

impl<T: Sized + Serialize> JSON<T> {
    pub fn ok(value: T) -> Result<Self> {
        Ok(JSON(value))
    }
    pub fn err<U: Display>(code: u16, message: U) -> Result<Self> {
        Error::try_new(code, message)
    }
}

impl<'a, T: Sized + Serialize + Deserialize<'a>> JSON<T> {
    pub fn parse<Str: ToString>(value: Str) -> Result<JSON<T>> {
        let value = value.try_to_string()?;
        match serde_json::from_str(Box::leak(value.into_boxed_str())) {
            Ok(result) => Ok(JSON(result)),
            Err(err) => Error::try_new(500, format!("Failed to deserialize json: {}", err)),
        }
    }
}

impl<T: Sized + Serialize> From<T> for JSON<T> {
    fn from(value: T) -> Self {
        JSON(value)
    }
}

impl<T: Sized + Serialize> Responder for JSON<T> {
    fn into_response(self) -> std::result::Result<(String, bytes::Bytes), Error> {
        match serde_json::to_string(&self.0) {
            Ok(json) => Ok(("application/json".to_string(), bytes::Bytes::from(json))),
            Err(_) => Error::try_new(500, "Failed to serialize json".to_string()),
        }
    }
}

pub struct HTML<T: ToString>(T);
impl<T: ToString> HTML<T> {
    pub fn ok(value: T) -> Result<Self> {
        Ok(HTML(value))
    }
    pub fn err<U: Display>(code: u16, message: U) -> Result<Self> {
        Error::try_new(code, message)
    }
}

impl<T: ToString> From<T> for HTML<T> {
    fn from(value: T) -> Self {
        HTML(value)
    }
}

impl<T: ToString> Responder for HTML<T> {
    fn into_response(self) -> std::result::Result<(String, bytes::Bytes), Error> {
        self.0
            .try_to_string()
            .map(|s| ("text/html".to_string(), bytes::Bytes::from(s)))
    }
}

pub struct File(pub String);
impl File {
    pub fn ok<T: Display>(value: T) -> Result<Self> {
        Ok(File(value.to_string()))
    }
    pub fn err<U: Display>(code: u16, message: U) -> Result<Self> {
        Error::try_new(code, message)
    }
}

impl From<&str> for File {
    fn from(value: &str) -> Self {
        File(PathBuf::from(ROOT).join(value).display().to_string())
    }
}

impl Responder for File {
    fn into_response(self) -> std::result::Result<(String, bytes::Bytes), Error> {
        let path = PathBuf::from(ROOT).join(self.0.clone());

        match fs::read_to_string(path.clone()) {
            Ok(s) => {
                let ct = match path.extension().unwrap().to_str().unwrap() {
                    "json" => "application/json",
                    "html" | "htm" => "text/html",
                    _ => "text/plain",
                };
                Ok((ct.to_string(), bytes::Bytes::from(s)))
            }
            Err(_) => Error::try_new(404, format!("Could not read file: {:?}", self.0)),
        }
    }
}

pub trait ToString {
    fn try_to_string(&self) -> std::result::Result<String, Error>;
}

impl<T: Display> ToString for T {
    fn try_to_string(&self) -> std::result::Result<String, Error> {
        Ok(self.to_string())
    }
}

impl ToString for File {
    fn try_to_string(&self) -> std::result::Result<String, Error> {
        match fs::read_to_string(self.0.clone()) {
            Ok(s) => Ok(s),
            Err(_) => Error::try_new(404, format!("Could not read file: {:?}", self.0)),
        }
    }
}
