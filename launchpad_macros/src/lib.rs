extern crate proc_macro;

mod args;
mod docs;
mod request;

use args::CatchArgs;
use docs::compile_docs;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

use request::{build_endpoint, request_expand, RequestArgs};

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
    let docs = format!(
        "#[doc=\"Catches {} errors and handles them\n\n{}\"]",
        match code.to_string().as_str() {
            "0" => "any",
            val => val,
        },
        compile_docs(&mut function)
    )
    .parse::<TokenStream2>()
    .unwrap();

    function.sig.ident = proc_macro2::Ident::new("__callback", function.sig.ident.span());
    function.vis = syn::Visibility::Inherited;

    quote! {
        #docs
        #[derive(Debug)]
        #[allow(non_camel_case_types)]
        #vis struct #name();

        #[allow(non_camel_case_types)]
        impl ::launchpad::router::endpoint::ErrorCatch for #name {
            fn execute(&self, code: u16, message: String) -> String {
                #function

                __callback(code, message)
            }

            #[inline]
            fn code(&self) -> u16 {
                #code
            }
        }
    }
    .into()
}

#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    let input: TokenStream2 = input.into();
    quote! {
        launchpad::response::HTML::of(html_to_string_macro::html! {
            #input
        })
    }
    .into()
}

// For testing wasm library
#[proc_macro_attribute]
pub fn main(_args: TokenStream, function: TokenStream) -> TokenStream {
    let function = parse_macro_input!(function as ItemFn);

    quote! {
        cfg_if::cfg_if! {
            if #[cfg(feature="client")] {
                #[wasm_bindgen::prelude::wasm_bindgen(start)]
                #function
            } else if #[cfg(feature="server")] {
                #function
            } else {
                fn main() {
                    panic!("Cannot run launchpad main without `server` or `client` features")
                }
            }
        }
    }
    .into()
}

#[proc_macro]
pub fn client(capture: TokenStream) -> TokenStream {
    let capture: TokenStream2 = capture.into();
    quote! {
        cfg_if::cfg_if! {
            if #[cfg(feature="client")] {
                #capture
            }
        }
    }
    .into()
}
