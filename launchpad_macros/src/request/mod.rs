use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{ItemFn, Visibility};

mod methods;
mod props;

use self::methods::build_method_comment_list;
pub use super::args::RequestArgs;
use props::compile_props;

use super::docs::compile_docs;
use methods::compile_methods_vec;

macro_rules! request_expand {
    ($name: ident, $method: ident) => {
        /// A request handler. Can optionally be given a uri or it can be provided
        /// in the `routes!` macro. This method only handles a single request method
        /// which is the same as the macro name.
        ///
        /// # Example
        /// ```
        /// use launchpad::router::prelude::*;
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
        #[proc_macro_error::proc_macro_error]
        #[proc_macro_attribute]
        pub fn $name(args: TokenStream, function: TokenStream) -> TokenStream {
            let mut args: RequestArgs = parse_macro_input!(args);
            args.methods.push(stringify!($method).to_string());

            // Get request doesn't get a body/content
            if stringify!($method) == "GET" {
                build_endpoint(args, parse_macro_input!(function), false)
            } else {
                build_endpoint(args, parse_macro_input!(function), true)
            }
        }
    };
}
pub(crate) use request_expand;

/// Update the function signature to better fit in its scope
fn update_function(function: &mut ItemFn) {
    function.vis = Visibility::Inherited;
    function.sig.ident = Ident::new("__endpoint", function.sig.ident.span());
}

/// Build the endpoint struct
pub fn build_endpoint(args: RequestArgs, mut function: ItemFn, include_data: bool) -> TokenStream {
    let (uri, path) = match &args.path {
        Some(p) => {
            let p = p.value().clone();
            (p.clone(), quote!(#p))
        }
        None => (String::new(), quote!(panic!("No path provided in macro"))),
    };

    // Collect information from function
    let name = function.sig.ident.clone();
    let (present, props) = compile_props(&function, &include_data);
    let methods = compile_methods_vec(&args);
    let docs = format!(
        "#[doc=\"{} endpoint for `{}`\n\n{}\"]",
        build_method_comment_list(&args),
        uri,
        compile_docs(&mut function)
    )
    .parse::<TokenStream2>()
    .unwrap();
    let visibility = function.vis.clone();

    update_function(&mut function);

    // Construct special endpoint props
    let state_type = match present.state {
        Some(ts) => ts,
        _ => quote!(::launchpad::router::request::Empty),
    };
    let content_local = match present.content {
        Some(ts) => ts,
        _ => quote!(),
    };
    let query_local = match present.query {
        Some(ts) => ts,
        _ => quote!(),
    };

    // Construct entire endpoint
    quote! {
        #docs
        #[derive(Debug)]
        #[allow(non_camel_case_types)]
        #visibility struct #name(pub std::sync::Mutex<::launchpad::router::request::State<#state_type>>);

        #[allow(non_camel_case_types)]
        impl ::launchpad::router::endpoint::Endpoint for #name {
            #[inline]
            fn methods(&self) -> Vec<hyper::Method> {
                #methods
            }

            #[inline]
            fn path(&self) -> String {
                String::from(#path)
            }

            fn execute(
                 &self,
                 uri: &hyper::Uri,
                 headers: &hyper::header::HeaderMap<hyper::header::HeaderValue>,
                 body: &bytes::Bytes
            ) -> ::launchpad::router::Response {
                #function

                let mut __lock_state = self.0.lock().unwrap();
                let mut __props = ::launchpad_props::props(&uri.path(), &self.path());
                #content_local
                #query_local

                ::launchpad::router::Response::from(__endpoint(#props))
            }
        }
    }
    .into()
}
