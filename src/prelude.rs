pub use crate::request::{Catch, Endpoint};
pub use crate::response::{template::TemplateEngine, Result, ToErrorResponse, ToResponse};
pub use crate::{context, group, response, template};
pub use serde_json::json;
pub use wayfinder_macros::{
    catch, connect, delete, get, head, html, options, patch, post, put, request, trace,
};

#[macro_export]
macro_rules! response {
    ($code: literal, $message: literal) => {
        Err(($code, $message.to_string()))
    };
    ($code: expr, $message: literal) => {
        Err(($code as u16, $message.to_string()))
    };
    ($result: expr) => {
        Ok($result)
    };
}

#[macro_export]
macro_rules! group {
    ($($items: expr),* $(,)?) => {
        vec![$(std::sync::Arc::new($items),)*]
    };
}

#[cfg(feature = "tera")]
#[macro_export]
macro_rules! tera {
    ($path: literal) => {
        ::wayfinder::response::Tera::new($path.to_string())
    };
    ($path: literal, $serializable: ident) => {
        ::wayfinder::response::Tera::from_struct($path.to_string(), $serializable)
    };
    ($path: literal, {$($key: literal => $value: expr),* $(,)?} $(,)?) => {
        ::wayfinder::response::Tera::new($path.to_string())
            $(.insert($key, &$value))*
    };
}

#[cfg(feature = "tera")]
pub use crate::tera;
