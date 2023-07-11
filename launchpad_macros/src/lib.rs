extern crate proc_macro;


use proc_macro2::TokenStream as TokenStream2;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    bracketed, parse::Parse, parse_macro_input, punctuated::Punctuated, FnArg, GenericArgument,
    Ident, ItemFn, LitStr, Pat, PatType, PathArguments, Result, Token, Type,
};

macro_rules! route_expand {
    ($name: ident, $method: ident) => {
        /// A request handler. Can optionally be given a uri or it can be provided
        /// in the `routes!` macro. This method only handles a single request method
        /// which is the same as the macro name.
        ///
        /// # Example
        /// ```
        /// use launchpad::prelude::*;
        ///
        /// #[get("/")]
        /// fn index() -> Result<&'static str> {
        ///     Ok("Hello World")
        /// }
        ///
        /// #[post]
        /// fn data() -> Result<&'static str> {
        ///     Ok("Home")
        /// }
        /// ```
        #[proc_macro_attribute]
        pub fn $name(args: TokenStream, function: TokenStream) -> TokenStream {
            let func: ItemFn = parse_macro_input!(function);
            let mut args: Args = parse_macro_input!(args as Args);
            args.methods.push(stringify!($method).to_string());

            build_endpoint(args, func)
        }
    };
}

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
    // assert!(!args.is_empty(), "requires at least a path argument");
    let func: ItemFn = parse_macro_input!(function);
    let args: Args = parse_macro_input!(args as Args);

    build_endpoint(args, func)
}

struct Args {
    path: Option<LitStr>,
    methods: Vec<String>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            path: None,
            methods: vec!["GET".to_string()],
        }
    }
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut path = None;
        if input.peek(LitStr) {
            path = Some(input.parse::<LitStr>()?);
            let _: Result<Token![,]> = input.parse();
        }

        let mut methods = Vec::new();
        if input.peek(Ident) {
            let next: Ident = input.parse()?;
            if next != "methods" {
                return Err(input.error("Unkown argument"));
            }

            let _: Token![=] = input.parse()?;
            let list;
            bracketed!(list in input);

            let req_methods = Punctuated::<Ident, Token![,]>::parse_terminated(&list)?;
            methods = req_methods
                .into_iter()
                .map(|m| m.to_string().to_uppercase())
                .collect()
        }

        Ok(Args { path, methods })
    }
}

/// Parse the function arguments and return a vector of types
fn parse_props(function: ItemFn) -> Vec<(String, Type)> {
    function
        .sig
        .inputs
        .into_iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(PatType { ty, pat, .. }) => {
                let name = match *pat {
                    Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                    _ => {
                        panic!("Expected named argument")
                    }
                };
                Some((name, *ty))
            }
            FnArg::Receiver(_) => None,
        })
        .collect::<Vec<(String, Type)>>()
}

fn get_state(prop: Type) -> Option<(bool, Type)> {
    if let Type::Reference(r) = &prop {
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
                    return Some((
                        match r.mutability {
                            Some(_) => true,
                            None => false,
                        },
                        elem,
                    ));
                }
            }
        }
    }
    None
}
/// Check if the function has a state struct
fn has_state(props: &mut Vec<(String, Type)>) -> Option<(bool, Type)> {
    let results = props
        .iter()
        .map(|s| get_state(s.1.clone()))
        .collect::<Vec<Option<(bool, Type)>>>();
    let pos = results.iter().position(|s| matches!(s, Some(_)));

    match pos {
        Some(index) => {
            props.remove(index);
            results[index].clone()
        }
        _ => None,
    }
}

/// Build the endpoint struct
fn build_endpoint(args: Args, function: ItemFn) -> TokenStream {
    let (_uri, path) = match args.path {
        Some(p) => {
            let p = p.value().clone();
            (p.clone(), quote!(String::from(#p)))
        }
        None => (
            String::new(),
            quote!(panic!("No path provided in macro. Please specify a path.")),
        ),
    };

    let name = function.sig.ident.clone();
    let mut props = parse_props(function.clone());

    let methods = args
        .methods
        .iter()
        .map(|m| format!("hyper::Method::{}", m))
        .collect::<Vec<String>>()
        .join(", ");

    let methods: TokenStream2 = format!("vec![{}]", methods)
        .parse::<TokenStream>()
        .unwrap()
        .into();

    let state = has_state(&mut props);

    let (stype, state) = match state {
        Some((mutable, elem)) => (
            quote!(#elem),
            match mutable {
                true => quote!(&mut *__lock_state),
                _ => quote!(&*__lock_state),
            },
        ),
        _ => (quote!(launchpad::state::Empty), quote!()),
    };

    let props = match props.len() > 0 {
        true => {
            let p: TokenStream2 = props
                .iter()
                .map(|f| format!("__props.get(\"{}\").unwrap().into()", f.0))
                .collect::<Vec<String>>()
                .join(", ")
                .parse::<TokenStream>()
                .unwrap()
                .into();

            if state.is_empty() {
                p
            } else {
                quote!(#state, #p)
            }
        }
        false => state,
    };

    let call = quote!(
        let mut __lock_state = self.0.lock().unwrap();
        let __props = launchpad_uri::props(&request.uri().path(), &self.path());

        match #name(#props) {
            Ok(__data) => launchpad::Response::from(__data),
            Err(__code) => launchpad::Response::from(__code),
        }
    );

    // TODO: Parse uri props and compare with method props
    // Ensure the types are the same

    quote! {
         #[derive(Debug)]
         #[allow(non_camel_case_types)]
         struct #name(std::sync::Mutex<launchpad::state::State<#stype>>);

         #[allow(non_camel_case_types)]
         impl launchpad::endpoint::Endpoint for #name {
             fn methods(&self) -> Vec<hyper::Method> {
                 #methods
             }

             fn path(&self) -> String {
                 #path
             }

             fn call(&self, request: hyper::Request<hyper::body::Incoming>) -> launchpad::Response {
                 #function

                 #call
             }
         }
    }
    .into()
}

// All specific request method varients
route_expand!(get, GET);
route_expand!(post, POST);
route_expand!(delete, DELETE);
route_expand!(put, PUT);
route_expand!(options, OPTIONS);
route_expand!(head, HEAD);
route_expand!(trace, TRACE);
route_expand!(connect, CONNECT);
route_expand!(patch, PATCH);
