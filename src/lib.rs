pub(crate) mod errors;
mod router;
mod server;

pub mod prelude;
pub mod request;
pub mod response;
pub mod support;
pub mod uri;

pub use errors::StatusCode;
pub use router::Router;
pub use server::Server;

/// Re-export needed dependencies for macros
pub mod bump {
    pub use bytes;
    pub use http_body_util;
    pub use hyper;
    pub use serde;
    pub use serde_json;
    pub use tokio;
}

pub use tela_macros::main;

pub trait StripPath {
    fn norm_strip_slashes(self) -> Self;
}

impl StripPath for String {
    fn norm_strip_slashes(mut self) -> Self {
        self = self.replace("\\", "/").replace("//", "/");
        if self.starts_with("/") {
            self = (&self[1..]).to_string();
        }
        if self.ends_with("/") {
            self = (&self[..self.len() - 1]).to_string();
        }
        self
    }
}
