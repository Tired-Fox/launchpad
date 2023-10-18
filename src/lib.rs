mod types;

pub mod prelude;

pub mod body;
pub mod client;
pub mod cookie;
pub mod error;
pub mod request;
pub mod response;
pub mod server;

pub use request::Request;
pub use response::Response;
pub use types::*;

pub mod external {
    pub use http_body_util;
    pub use hyper;
}

pub use hyper;
use hyper::body::Bytes;
pub use serde::{Deserialize, Serialize};
pub use tokio::main;

pub use tela_macros::debug_release as dbr;

pub type Empty = http_body_util::Empty<Bytes>;
pub type Full = http_body_util::Full<Bytes>;
