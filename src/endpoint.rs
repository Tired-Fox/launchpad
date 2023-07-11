use std::{fmt::Debug, collections::HashMap};

pub use hyper::Method;

use super::Response;

// PERF: Move to own file
pub struct Context;

/// Trait for anything that can be transformed into Bytes for a successfull
/// hyper response.
pub trait Responder {
    fn into_response(self) -> bytes::Bytes;
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
    fn call(&self, request: hyper::Request<hyper::body::Incoming>) -> Response;
    fn path(&self) -> String;
    fn methods(&self) -> Vec<Method>;
    fn props(&self, uri: String) -> HashMap<String, launchpad_uri::Prop> {
       launchpad_uri::props(&uri, &self.path())
    }
}
