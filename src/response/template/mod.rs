pub mod hbs;
pub mod ttera;
use std::{collections::BTreeMap, marker::PhantomData};

#[cfg(feature = "handlebars")]
pub use hbs::Handlebars;
#[cfg(feature = "tera")]
pub use ttera::Tera;

use super::{Result, ToErrorResponse, ToResponse};

#[macro_export]
macro_rules! context {
    ($($key: ident: $value: expr),* $(,)?) => {
        std::collections::BTreeMap::<String, serde_json::Value>::from([
            $((stringify!($key).to_string(), serde_json::to_value(&$value).unwrap()),)*
        ])
    };
    (...$spread: expr, $($key: ident: $value: expr),* $(,)?) => {
        $crate::response::template::extend_context($spread, [
                $((stringify!($key).to_string(), serde_json::to_value(&$value).unwrap()),)*
        ])
    };
}

#[macro_export]
macro_rules! template {
    ($path: literal) => {
       crate::response::Template::new($path, context!{})
    };
    ($path: literal, { $($context: tt)* } $(,)?) => {
       crate::response::Template::new($path, context!{$($context)*})
    };
    ($path: literal, $context: ident $(,)?) => {
       crate::response::Template::new($path, $context)
    };
}

pub trait TemplateEngine {
    fn parse_path(path: &str) -> String;
    fn context() -> BTreeMap<String, serde_json::Value>;
    fn init<T: Into<String>>(path: T, globals: BTreeMap<String, serde_json::Value>);
    fn globals() -> BTreeMap<String, serde_json::Value>;
    fn render(path: String, context: BTreeMap<String, serde_json::Value>) -> Result<String>;
}

pub trait TreeToTemplateContext {
    type Return;
    fn to_context(value: BTreeMap<String, serde_json::Value>) -> Self::Return;
}

pub struct Template<ENGINE: TemplateEngine>(
    pub String,
    pub BTreeMap<String, serde_json::Value>,
    PhantomData<ENGINE>,
);

impl<ENGINE: TemplateEngine> Template<ENGINE> {
    pub fn new<T: Into<String>>(path: T, context: BTreeMap<String, serde_json::Value>) -> Self {
        Template(path.into(), context, PhantomData)
    }

    pub fn render(self) -> Result<String> {
        ENGINE::render(ENGINE::parse_path(&self.0), self.1)
    }
}

impl<T: TemplateEngine> ToResponse for Template<T> {
    fn to_response(
        self,
        _method: &hyper::Method,
        _uri: &hyper::Uri,
        _body: String,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
        self.render().map(|text| {
            hyper::Response::builder()
                .status(200)
                .body(http_body_util::Full::new(bytes::Bytes::from(text)))
                .unwrap()
        })
    }
}

impl<T: TemplateEngine> ToErrorResponse for Template<T> {
    fn to_error_response(
        self,
        _code: u16,
        _reason: String,
    ) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
        self.render().map(|text| {
            hyper::Response::builder()
                .status(200)
                .body(http_body_util::Full::new(bytes::Bytes::from(text)))
                .unwrap()
        })
    }
}

/// Used to extend a BTreeMap<String, serde_json::Value> with an array of values
/// of equivelant types.
pub fn extend_context<const SIZE: usize>(
    mut map: BTreeMap<String, serde_json::Value>,
    values: [(String, serde_json::Value); SIZE],
) -> BTreeMap<String, serde_json::Value> {
    map.append(&mut BTreeMap::from(values));
    map
}
