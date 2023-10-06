extern crate proc_macro;

mod debug_release;
mod fetch;

use quote::quote;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

use debug_release::DebugRelease;
use fetch::Fetch;

/// Generate a http request.
///
/// This method is async which means for a value to be returned `await` must be used on the macros
/// result.
#[proc_macro_error]
#[proc_macro]
pub fn fetch(input: TokenStream) -> TokenStream {
    let fetch = parse_macro_input!(input as Fetch);
    quote!(#fetch).into()
}

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
