use std::{collections::HashMap, fmt::Debug};

pub use hyper::Method;

use super::{Error, Response};

/// Trait for anything that can be transformed into Bytes for a successfull
/// hyper response.
pub trait Responder {
    fn into_response(self) -> std::result::Result<(String, bytes::Bytes), Error>;
}

/// The contracts and layout for the structs created with the request macros
///
/// # Example
/// ```
/// use launchpad::prelude::*;
///
/// #[get("/")]
/// fn home() -> Result<&'static str> {
///     Ok("Hello, world!")
/// }
/// ```
///
/// Would yield
///
/// ```
/// #[derive(Debug)]
/// #[allow(non_camel_case_types)]
/// struct home(std::sync::Mutex<launchpad::state::State<#stype>>);
///
/// #[allow(non_camel_case_types)]
/// impl launchpad::endpoint::Endpoint for home {
///     fn methods(&self) -> Vec<hyper::Method> {
///        vec![hyper::Method::GET]
///     }
///
///     fn path(&self) -> String {
///         String::from("/")
///     }
///
///     fn call(&self) -> launchpad::Response {
///         fn home() -> Result<&'static str> {
///             Ok("Hello, world!")
///         }
///
///         match home() {
///             Ok(__data) => launchpad::Response::from(__data),
///             Err(__code) => launchpad::Response::from(__code),
///         }
///     }
/// }
/// ```
/// ```
/// ```
pub trait Endpoint: Debug + Sync + Send {
    fn execute(
        &self,
        uri: &hyper::Uri,
        headers: &hyper::header::HeaderMap<hyper::header::HeaderValue>,
        body: &bytes::Bytes,
    ) -> Response;
    fn path(&self) -> String;
    fn methods(&self) -> Vec<Method>;
    fn props(&self, uri: String) -> HashMap<String, launchpad_props::Prop> {
        launchpad_props::props(&uri, &self.path())
    }
}

pub trait ErrorCatch: Debug + Sync + Send {
    fn execute(&self, code: u16, message: String) -> String;
    fn code(&self) -> u16;
}
