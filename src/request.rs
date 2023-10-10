use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Display;

use http_body_util::BodyExt;
use hyper::Version;
use hyper::{
    body::{Body, Bytes, Incoming},
    Request as HttpRequest,
};
use serde::Deserialize;

use crate::body::{IntoBody, ParseBody};
use crate::error::Error;
use crate::Full;

pub struct Builder {
    uri: String,
    headers: HashMap<String, String>,
    method: String,
    version: Version,
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            uri: String::new(),
            headers: HashMap::new(),
            method: String::from("GET"),
            version: Version::HTTP_10,
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Builder::default()
    }

    pub fn uri<T>(mut self, uri: T) -> Self
    where
        T: ToString,
    {
        self.uri = uri.to_string().replace(" ", "%20");
        self
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: ToString,
        V: Display,
    {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn method<M>(mut self, method: M) -> Self
    where
        M: ToString,
    {
        self.method = method.to_string();
        self
    }

    pub fn version(mut self, version: f32) -> Self {
        if version == 0.9 {
            self.version = Version::HTTP_09;
        } else if version == 1.0 {
            self.version = Version::HTTP_10;
        } else if version == 1.1 {
            self.version = Version::HTTP_11;
        } else if version == 2.0 {
            self.version = Version::HTTP_2;
        } else if version == 3.0 {
            self.version = Version::HTTP_3;
        }
        self
    }

    pub fn body<B, T>(self, body: T) -> HttpRequest<B>
    where
        B: Body<Data = Bytes, Error = Infallible>,
        T: IntoBody<B>,
    {
        let mut builder = HttpRequest::builder()
            .uri(self.uri)
            .method(self.method.as_str())
            .version(self.version);

        for (key, value) in self.headers.iter() {
            builder = builder.header(key, value);
        }

        builder.body(body.into_body()).unwrap()
    }
}

pub struct Request(HttpRequest<Incoming>);

impl From<HttpRequest<Incoming>> for Request {
    fn from(value: HttpRequest<Incoming>) -> Self {
        Request(value)
    }
}

impl From<Request> for HttpRequest<Incoming> {
    fn from(value: Request) -> Self {
        value.0
    }
}

impl<'r> ParseBody<'r> for Request {
    fn text(
        self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Error>> + Send>> {
        Box::pin(async move {
            String::from_utf8(self.0.collect().await.unwrap().to_bytes().to_vec())
                .map_err(Error::from)
        })
    }

    fn raw(self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<u8>> + Send>> {
        Box::pin(async move { self.0.collect().await.unwrap().to_bytes().to_vec() })
    }
}

impl<'r> Request {
    pub fn new(req: HttpRequest<Incoming>) -> Self {
        Request(req)
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn uri(&self) -> String {
        self.0.uri().to_string()
    }

    pub fn query<T: Deserialize<'r>>(&self) -> Result<T, String> {
        match self.0.uri().query() {
            Some(query) => serde_qs::from_str::<T>(Box::leak(String::from(query).into_boxed_str()))
                .map_err(|e| e.to_string()),
            None => Err("No query available to parse".to_string()),
        }
    }
}
