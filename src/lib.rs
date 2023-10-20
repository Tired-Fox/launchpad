mod types;

pub mod prelude;

pub mod body;
pub mod client;
pub mod cookie;
pub mod error;
pub mod request;
pub mod response;
pub mod server;

use std::future::Future;

pub use request::Request;
pub use response::Response;
pub use types::*;

pub use http_body_util::{Empty, Full};
pub use hyper::{body::Bytes, Response as HttpResponse};

/// A generic filler type for typing IntoBody types.
///
/// Most useful for cases where `None` is needed. Ex: `None::<Fill>`.
pub type Fill = http_body_util::Empty<Bytes>;

pub use serde::{Deserialize, Serialize};

pub use tela_macros::{debug_release as dbr, main};

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
