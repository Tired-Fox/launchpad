extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod html;
use html::Parser;

#[proc_macro_error]
#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    let segment = parse_macro_input!(input as Parser);
    quote!(#segment).into()
}

#[proc_macro_derive(Prop)]
pub fn prop(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input.clone()).unwrap();

    let name = ast.ident;

    let params = ast.generics.params;
    let where_clause = ast.generics.where_clause;

    quote! {
        impl<#params> tela_html::Prop for #name<#params> #where_clause {}
    }
    .into()
}
