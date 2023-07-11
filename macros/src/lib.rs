extern crate proc_macro;

use proc_macro2::TokenStream as TokenStream2;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    bracketed, parse::Parse, parse_macro_input, punctuated::Punctuated, FnArg, GenericArgument,
    Ident, ItemFn, LitStr, PatType, PathArguments, Result, Token, Type,
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

/// Check if the function has a state struct
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

/// Build the endpoint struct
fn build_endpoint(args: Args, function: ItemFn) -> TokenStream {
    let path = match args.path {
        Some(p) => {
            let p = p.value();
            quote!(String::from(#p))
        }
        None => quote!(panic!("No path provided in macro. Please specify a path.")),
    };

    let name = function.sig.ident.clone();
    let props = parse_props(function.clone());

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

    let (state, state_mutable, elem) = has_state(&props);
    let (stype, state) = match state {
        true => {
            let elem = elem.unwrap();
            (
                quote!(#elem),
                match state_mutable {
                    true => quote!(
                        let mut __lock_state = self.0.lock().unwrap();
                        match #name(&mut *__lock_state) {
                            Ok(__data) => launchpad::Response::from(__data),
                            Err(__code) => launchpad::Response::from(__code),
                        }
                    ),
                    _ => quote!(
                        let mut __lock_state = self.0.lock().unwrap();
                        match #name(&*__lock_state) {
                            Ok(__data) => launchpad::Response::from(__data),
                            Err(__code) => launchpad::Response::from(__code),
                        }
                    ),
                },
            )
        }
        _ => (
            quote!(launchpad::state::Empty),
            quote!(
                match #name() {
                    Ok(__data) => launchpad::Response::from(__data),
                    Err(__code) => launchpad::Response::from(__code),
                }
            ),
        ),
    };

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

             fn call(&self) -> launchpad::Response {
                 #function

                 #state
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
