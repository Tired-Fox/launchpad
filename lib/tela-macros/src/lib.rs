extern crate proc_macro;

mod debug_release;
mod fetch;

use quote::quote;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use syn::{parse_macro_input, ItemFn};

use debug_release::DebugRelease;
use fetch::Fetch;

/// Generate a http request.
///
/// This method is async which means for a value to be returned `await` must be used on the macros
/// result.
///
/// The first argument is always the url to use. This can be a static str or a String and it can
/// also be a variable.
///
/// The other arguments include: `method`, `headers`, and `body`. The `headers` label is not
/// required. All of these arguments are optional.
///
/// - `method`: Any method type for an http request. This can be uppercase or lowercase.
/// - `headers`: Map of header name to it's value. The header name can be a snake case identifier
/// that is automatically translated to upper cabob case. Ex: `content_type` == `Content-Type`
/// - `body`: Assign a body value. This can be anything that implements `tela::IntoBody`.
///
/// # Example
/// ```
///     fetch! {
///         "example.com/",
///         method: post,
///         headers: {
///             content_type: "text/html"
///         },
///         body: html::new! {
///             <div>"Sample html"</div>
///         }
///     }
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn fetch(input: TokenStream) -> TokenStream {
    let fetch = parse_macro_input!(input as Fetch);
    quote!(#fetch).into()
}

/// Useful for code that is different between debugging and release.
///
/// First argument is debug and the second is release. These two arguments
/// can also be labeled with `d`, `dbg`, and `debug` for the debug argument.
/// Release tags include: `r`, `rls`, and `release`.
///
/// # Example
/// ```
///     debug_release!("debug", "release")
///     // or
///     debug_release!(dbg: "debug", rls: "release")
///     // etc...
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn debug_release(input: TokenStream) -> TokenStream {
    #[allow(unused_variables)]
    let DebugRelease(debug, release) = parse_macro_input!(input as DebugRelease);

    #[cfg(debug_assertions)]
    return quote!(#debug).into();
    #[cfg(not(debug_assertions))]
    return quote!(#release).into();
}

/// Wraps a function in a tokio runtime as an async entry point.
///
/// Under the hood it uses `tela::runtime_entry(async {})`
#[proc_macro_error]
#[proc_macro_attribute]
pub fn main(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut dt: ItemFn = parse_macro_input!(input as ItemFn);

    let _body = dt.block;
    let _ident = dt.sig.ident.clone();
    match dt.sig.asyncness {
        Some(a) => a,
        None => abort!(dt.sig.fn_token, "Expected function signature to be async"),
    };
    dt.sig.asyncness = None;
    let _sig = dt.sig;

    quote! {
        #_sig {
            tela::runtime_entry(async {
                #_body
            })
        }
    }
    .into()
}
