#![doc = include_str!("./docs/root.md")]

use std::future::Future;

pub use http_body_util::{Empty, Full};
pub use hyper::{body::Bytes, Response as HttpResponse};
pub use serde::{Deserialize, Serialize};
pub use tela_macros::{debug_release as dbr, main};

pub use request::Request;
pub use response::Response;

pub mod extract;
pub mod prelude;
pub mod body;
pub mod client;
pub mod cookie;
pub mod error;
pub mod request;
pub mod response;
pub mod server;
pub mod sync;

/// A generic filler type for typing IntoBody types.
///
/// Most useful for cases where `None` is needed. Ex: `None::<Fill>`.
pub type Fill = http_body_util::Empty<Bytes>;

/// The async entrypoint for tela.
///
/// This ends up being a wrapper around building a tokio multi threaded runtime. This method can
/// either be invoked manually or by applying the macro to a method with `#[tela::main]`
///
/// # Example
/// ```
/// #[tela::main]
/// async fn main() {
///     //...
/// }
/// ```
/// or
/// ```
/// fn main() {
///     tela::runtime_entry(async {
///         //...
///     })
/// }
/// ```
pub fn runtime_entry<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}

pub mod html {
    pub use tela_html::html as new;
    pub use tela_html::prelude::*;
    pub use tela_html::props;

    pub use crate::_html as into;
    use crate::response::Html;

    #[doc = "Same as html::new! except `into` is automatically called on the result"]
    #[macro_export]
    macro_rules! _html {
        ($($html: tt)*) => {
            $crate::html::new! {
                $($html)*
            }.into()
        };
        ($type: ty as $($html: tt)*) => {
            Into::<$type>::into($crate::html::new! {
                $($html)*
            })
        };
    }

    impl From<Element> for Html<String> {
        fn from(value: Element) -> Self {
            Html(value.to_string())
        }
    }
}

pub mod json {
    pub use serde_json::json as new;
    pub use serde_json::Value;

    pub use crate::_json as into;

    #[doc = "Same as json::new! except `into` is automatically called on the result"]
    #[macro_export]
    macro_rules! _json {
        ($type: ty as {$($json: tt)*}) => {
            Into::<$type>::into($crate::json::new!({$($json)*}))
        };
        ({$($json: tt)*}) => {
            $crate::json::new!({$($json)*}).into()
        };
    }
}
