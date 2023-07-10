extern crate proc_macro;

use proc_macro2::TokenStream as TokenStream2;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, FnArg, GenericArgument, ItemFn, LitStr, PatType, PathArguments, Type
};

macro_rules! route_expand {
    ($name: ident, $method: ident) => {
        #[proc_macro_attribute]
        pub fn $name(args: TokenStream, function: TokenStream) -> TokenStream {
            assert!(!args.is_empty(), "requires a path argument");
            let func: ItemFn = parse_macro_input!(function);
            let args: Args = parse_macro_input!(args as Args);
        
            build_endpoint(args, func, quote!(vec![hyper::Method::$method]))
        }
    };
}

struct Args {
    path: LitStr,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            Ok(Args {
                path: input.parse()?,
            })
        } else {
            Err(input.error("Expected path string"))
        }
        // let vars = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
    }
}

fn parse_props(function: ItemFn) -> Vec<Type> {
    function
        .sig
        .inputs
        .into_iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(PatType { ty, .. }) => Some(*ty),
            FnArg::Receiver(_) => None,
        })
        .collect::<Vec<Type>>()
}
fn has_state(props: &Vec<Type>) -> (bool, bool, Option<Type>) {
    for prop in props.iter() {
        if let Type::Reference(r) = prop {
            if let Type::Path(path) = &*r.elem {
                if let Some(seg) = path.path.segments.last() {
                    let elem = match &seg.arguments {
                        PathArguments::AngleBracketed(brackets) => {
                            if brackets.args.len() == 1 {
                                match &brackets.args[0] {
                                    GenericArgument::Type(t) => t.clone(),
                                    _ => panic!("Expected state type to be a type"),
                                }
                            } else {
                                panic!("Expected one state type")
                            }
                        }
                        _ => panic!("Expected State generic type"),
                    };
                    if seg.ident.to_string() == "State".to_string() {
                        return (
                            true,
                            match r.mutability {
                                Some(_) => true,
                                None => false,
                            },
                            Some(elem),
                        );
                    }
                }
            }
        }
    }
    (false, false, None)
}

fn build_endpoint(args: Args, function: ItemFn, methods: TokenStream2) -> TokenStream {
    let path = args.path.value();
    let name = function.sig.ident.clone();
    let props = parse_props(function.clone());

    let (state, state_mutable, elem) = has_state(&props);
    let (stype, state) = match state {
        true => {
            let elem = elem.unwrap();
            (
                quote!((std::sync::Mutex<launchpad::v2::state::State<#elem>>)),
                match state_mutable {
                    true => quote!(
                        let mut lock_state = self.0.lock().unwrap();
                        match #name(&mut *lock_state) {
                            Ok(data) => launchpad::v2::Response::from(data),
                            Err(code) => launchpad::v2::Response::from(code),
                        }
                    ),
                    _ => quote!(
                        let mut lock_state = self.0.lock().unwrap();
                        match #name(&*lock_state) {
                            Ok(data) => launchpad::v2::Response::from(data),
                            Err(code) => launchpad::v2::Response::from(code),
                        }
                    ),
                },
            )
        }
        _ => (quote!(), quote!(
            match #name() {
                Ok(data) => launchpad::v2::Response::from(data),
                Err(code) => launchpad::v2::Response::from(code),
            }
        )),
    };

    quote! {
         #[derive(Debug)]
         #[allow(non_camel_case_types)]
         struct #name #stype;

         #[allow(non_camel_case_types)]
         impl Endpoint for #name {
             fn methods(&self) -> Vec<hyper::Method> {
                 #methods
             }

             fn path(&self) -> String {
                 String::from(#path)
             }

             fn call(&self) -> Response {
                 #function

                 #state
             }
         }
    }
    .into()
}

route_expand!(get, GET);
route_expand!(post, POST);
route_expand!(delete, DELETE);
route_expand!(put, PUT);
