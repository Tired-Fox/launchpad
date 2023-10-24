use std::{convert::Infallible, path::PathBuf};

use async_trait::async_trait;
use http_body_util::{Empty, Full};
use hyper::{
    body::{Body, Bytes},
    StatusCode,
};
use serde::Deserialize;
use serde_json::Value;
use tela_html::Element;

use crate::{error::Error, Html};

/// Parse the body into repspective types.
///
/// The only required method to implement is `text` as all other types
/// are parsed from the result of that type
#[async_trait]
pub trait ParseBody<'r> {
    /// Parse the body as a form/query string.
    async fn form<O>(self) -> Result<O, Error>
    where
        O: Deserialize<'r>,
        Self: Sized + 'static + Send,
    {
        let content = self.text().await.unwrap();
        serde_qs::from_str(Box::leak(content.clone().into_boxed_str())).map_err(|e| {
            Error::from((StatusCode::INTERNAL_SERVER_ERROR, e, {
                #[cfg(debug_assertions)]
                {
                    Html(content.to_string())
                }
                #[cfg(not(debug_assertions))]
                {
                    Html(String::new())
                }
            }))
        })
    }

    /// Parse the body as a json string.
    async fn json<O>(self) -> Result<O, Error>
    where
        O: Deserialize<'r>,
        Self: Sized + 'static + Send,
    {
        let content = self.text().await.unwrap();
        serde_json::from_str(Box::leak(content.clone().into_boxed_str())).map_err(|e| {
            Error::from((StatusCode::INTERNAL_SERVER_ERROR, e, {
                #[cfg(debug_assertions)]
                {
                    Html(content.to_string())
                }
                #[cfg(not(debug_assertions))]
                {
                    Html(String::new())
                }
            }))
        })
    }

    /// Get the body as a raw String.
    async fn text(self) -> Result<String, Error>;

    /// Get the body as raw bytes.
    async fn raw(self) -> Vec<u8>;

    /// Parse the body as a top level primitive (basic) type.
    async fn base<O>(self) -> Result<O, Error>
    where
        O: Deserialize<'r>,
        Self: Sized + 'static + Send,
    {
        let content = self.text().await.unwrap();
        serde_plain::from_str::<O>(Box::leak(content.clone().into_boxed_str())).map_err(|e| {
            Error::from((StatusCode::INTERNAL_SERVER_ERROR, e, {
                #[cfg(debug_assertions)]
                {
                    Html(content.to_string())
                }
                #[cfg(not(debug_assertions))]
                {
                    Html(String::new())
                }
            }))
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

impl IntoBody<Full<Bytes>> for Full<Bytes> {
    fn into_body(self) -> Full<Bytes> {
        self
    }
}
impl IntoBody<Empty<Bytes>> for Empty<Bytes> {
    fn into_body(self) -> Empty<Bytes> {
        self
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

impl IntoBody<Full<Bytes>> for Value {
    fn into_body(self) -> Full<Bytes> {
        Full::new(Bytes::from(self.to_string()))
    }
}

impl IntoBody<Full<Bytes>> for Element {
    fn into_body(self) -> Full<Bytes> {
        Full::new(Bytes::from(self.to_string()))
    }
}

impl IntoBody<Full<Bytes>> for PathBuf {
    fn into_body(self) -> Full<Bytes> {
        match std::fs::read(self) {
            Ok(file) => Full::new(Bytes::from(file)),
            Err(e) => {
                eprintln!("Error while serving file: {}", e);
                Full::default()
            }
        }
    }
}
