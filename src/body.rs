use std::{convert::Infallible, future::Future, pin::Pin};

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
pub trait ParseBody<'r> {
    fn form<O>(self) -> Pin<Box<dyn Future<Output = Result<O, Error>> + Send>>
    where
        O: Deserialize<'r>,
        Self: Sized + 'static + Send,
    {
        Box::pin(async move {
            let content = self.text().await.unwrap();
            serde_qs::from_str(Box::leak(content.clone().into_boxed_str())).map_err(|e| {
                Error::from((StatusCode::INTERNAL_SERVER_ERROR, e, {
                    #[cfg(debug_assertions)]
                    {
                        Html(html_to_string_macro::html!(<pre>{content.to_string()}</pre>))
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        Html(String::new())
                    }
                }))
            })
        })
    }

    fn json<O>(self) -> Pin<Box<dyn Future<Output = Result<O, Error>> + Send>>
    where
        O: Deserialize<'r>,
        Self: Sized + 'static + Send,
    {
        Box::pin(async move {
            let content = self.text().await.unwrap();
            serde_json::from_str(Box::leak(content.clone().into_boxed_str())).map_err(|e| {
                Error::from((StatusCode::INTERNAL_SERVER_ERROR, e, {
                    #[cfg(debug_assertions)]
                    {
                        Html(html_to_string_macro::html!(<pre>{content.to_string()}</pre>))
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        Html(String::new())
                    }
                }))
            })
        })
    }

    fn text(self) -> Pin<Box<dyn Future<Output = Result<String, Error>> + Send>>;

    fn raw(self) -> Pin<Box<dyn Future<Output = Vec<u8>> + Send>>;

    fn multipart<O>(self) -> Pin<Box<dyn Future<Output = Result<O, Error>> + Send>>
    where
        O: Deserialize<'r>,
        Self: Sized + 'static + Send,
    {
        Box::pin(async move {
            let content = self.text().await.unwrap();
            serde_qs::from_str::<O>(Box::leak(content.clone().into_boxed_str())).map_err(|e| {
                Error::from((StatusCode::INTERNAL_SERVER_ERROR, e, {
                    #[cfg(debug_assertions)]
                    {
                        Html(html_to_string_macro::html!(<pre>{content.to_string()}</pre>))
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        Html(String::new())
                    }
                }))
            })
        })
    }

    fn primitive<O>(self) -> Pin<Box<dyn Future<Output = Result<O, Error>> + Send>>
    where
        O: Deserialize<'r>,
        Self: Sized + 'static + Send,
    {
        Box::pin(async move {
            let content = self.text().await.unwrap();
            serde_plain::from_str::<O>(Box::leak(content.clone().into_boxed_str())).map_err(|e| {
                Error::from((StatusCode::INTERNAL_SERVER_ERROR, e, {
                    #[cfg(debug_assertions)]
                    {
                        Html(html_to_string_macro::html!(<pre>{content.to_string()}</pre>))
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        Html(String::new())
                    }
                }))
            })
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
