extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, FnArg, Type, ItemFn, LitStr};

struct Args {
    path: LitStr,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            Ok(Args {
                path: input.parse()?
            })
        } else {
            Err(input.error("Expected path string"))
        }
        // let vars = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
    }
}

#[proc_macro_attribute]
pub fn get(args: TokenStream, function: TokenStream) -> TokenStream {
    let result = function.clone();
    assert!(!args.is_empty(), "requires an argument");
    let func: ItemFn = parse_macro_input!(function);
    let args: Args = parse_macro_input!(args as Args);

    println!("{:?}", args.path.value());
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

    // Request::new(vec![Method::Get], Arc::new(|req| None));
    // quote! { fn #func.}.into()
    result
}
