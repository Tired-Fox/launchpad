extern crate proc_macro;

mod request;
use proc_macro::TokenStream;
use syn::parse_macro_input;

use request::{build_endpoint, request_expand, Args};

/// Base request macro. It accepts a path and a list of request methods.
/// All request methods are valid for the endpoint and the path is optional.
///
/// # Example
/// ```
/// use launchpad::prelude::*;
///
/// #[request]
/// fn index() -> Result<&'static str> {}
///
/// #[request("/")]
/// fn data() -> Result<&'static str> {}
///
/// #[request("/", methods=[get, post, delete])]
/// fn delete() -> Result<&'static str> {}
///
/// #[request(methods=[get, post, delete])]
/// fn home() -> Result<&'static str> {}
/// ```
#[proc_macro_attribute]
pub fn request(args: TokenStream, function: TokenStream) -> TokenStream {
    build_endpoint(
        parse_macro_input!(args as Args),
        parse_macro_input!(function),
        true,
    )
}

// All specific request method varients
request_expand!(get, GET);
request_expand!(post, POST);
request_expand!(delete, DELETE);
request_expand!(put, PUT);
request_expand!(options, OPTIONS);
request_expand!(head, HEAD);
request_expand!(trace, TRACE);
request_expand!(connect, CONNECT);
request_expand!(patch, PATCH);
