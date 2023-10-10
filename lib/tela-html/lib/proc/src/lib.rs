extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::parse_macro_input;

mod html;

use crate::html::Segment;

#[proc_macro_error]
#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    let segment = parse_macro_input!(input as Segment);
    quote!(#segment).into()
}
