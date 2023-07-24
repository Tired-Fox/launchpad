extern crate proc_macro;

mod request;
mod router;
mod docs;
use docs::compile_docs;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use router::Routes;
use syn::{parse_macro_input, ItemFn};
use proc_macro_error::proc_macro_error;

use request::{build_endpoint, request_expand, RequestArgs};
use router::CatchArgs;

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
#[proc_macro_error]
#[proc_macro_attribute]
pub fn request(args: TokenStream, function: TokenStream) -> TokenStream {
    build_endpoint(
        parse_macro_input!(args as RequestArgs),
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

#[proc_macro_error]
#[proc_macro_attribute]
pub fn catch(args: TokenStream, function: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(function as ItemFn);
    let args = parse_macro_input!(args as CatchArgs);

    let name = function.sig.ident.clone();
    let vis = function.vis.clone();
    let code = args.code;
    let docs = format!("#[doc=\"Catches {} errors and handles them\n\n{}\"]", code, compile_docs(&mut function))
        .parse::<TokenStream2>().unwrap();

    function.sig.ident = proc_macro2::Ident::new("__callback", function.sig.ident.span());
    function.vis = syn::Visibility::Inherited;

    quote!{
        #docs
        #[derive(Debug)]
        #[allow(non_camel_case_types)]
        #vis struct #name();

        #[allow(non_camel_case_types)]
        impl ::launchpad::endpoint::ErrorCatch for #name {
            #[inline]
            fn execute(&self, message: String) -> String {
                #function

                __callback(self.code(), message)
            }

            #[inline]
            fn code(&self) -> u16 {
                #code
            }
        }
    }.into()
}

/// Build a router that handles requests to endpoints or errors
#[proc_macro_error]
#[proc_macro]
pub fn rts(tokens: TokenStream) -> TokenStream {
    let routes = parse_macro_input!(tokens as Routes);
    let endpoints = routes.endpoints.join(", ").parse::<TokenStream2>().unwrap();
    
    if routes.catches.len() > 0 {
        let catches = routes.catches.join(", ").parse::<TokenStream2>().unwrap();
        quote!{
            ::launchpad::router::Router::from((
                [#endpoints],
                [#catches]
            ))
        }.into()
    } else {
        quote!{ ::launchpad::router::Router::from([#endpoints]) }.into()
    }
}
