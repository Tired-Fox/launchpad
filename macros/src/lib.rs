extern crate proc_macro;
use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, punctuated::Punctuated, Ident, Token, FnArg, Type, ItemFn};

struct Args {
    vars: HashSet<Ident>,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            vars: vars.into_iter().collect(),
        })
    }
}

#[proc_macro_attribute]
pub fn get(args: TokenStream, function: TokenStream) -> TokenStream {
    assert!(!args.is_empty(), "requires an argument");
    let func: ItemFn = parse_macro_input!(function);
    let args: Vec<String> = parse_macro_input!(args as Args)
        .vars
        .iter()
        .map(|i| i.to_string())
        .collect();
    assert!(args.len() == 1, "only one argument is allowed");

    for arg in func.sig.inputs.iter() {
        match arg {
            FnArg::Typed(pat_type) => {
                match pat_type.ty.as_ref() {
                    Type::Paren(paren) => {
                        println!("{:?}", paren.paren_token.span);
                    },
                    _ => {}
                }
            },
            FnArg::Receiver(_) => {}
        }
    }

    quote! {Request::new(vec![Method::Get], Arc::new(|req| None))}.into()
}
