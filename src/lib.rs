pub mod prelude;

pub mod body;
pub mod request;
pub mod response;

pub mod client;
pub mod error;
pub mod server;

use hyper::body::Bytes;
pub use request::Request;
pub use response::Response;

pub use tela_html as html;

pub use hyper;
pub use tokio::main;

pub mod external {
    pub use http_body_util;
    pub use hyper;
}

pub use serde::{Deserialize, Serialize};

#[cfg(feature = "macros")]
pub use tela_macros::debug_release as dbr;

pub type Empty = http_body_util::Empty<Bytes>;
pub type Full = http_body_util::Full<Bytes>;
