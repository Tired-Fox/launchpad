use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ItemFn;

mod args;
mod props;

pub use args::Args;
use props::compile_props;

macro_rules! request_expand {
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
            let mut args: Args = parse_macro_input!(args as Args);
            args.methods.push(stringify!($method).to_string());

            build_endpoint(args, parse_macro_input!(function))
        }
    };
}
pub(crate) use request_expand;

fn build_methods(args: &Args) -> TokenStream2 {
    format!(
        "vec![{}]",
        args.methods
            .iter()
            .map(|m| format!("hyper::Method::{}", m))
            .collect::<Vec<String>>()
            .join(", ")
    )
    .parse::<TokenStream>()
    .unwrap()
    .into()
}

/// Build the endpoint struct
pub fn build_endpoint(args: Args, function: ItemFn) -> TokenStream {
    let (_uri, path) = match &args.path {
        Some(p) => {
            let p = p.value().clone();
            (p.clone(), quote!(String::from(#p)))
        }
        None => (String::new(), quote!(panic!("No path provided in macro"))),
    };

    let name = function.sig.ident.clone();
    let methods = build_methods(&args);
    let (present, props) = compile_props(&function);

    let props = quote!(#props);

    let state = match present.state {
        Some(ts) => ts,
        _ => quote!(::launchpad::arguments::Empty),
    };

    let data = match present.data {
        Some(ts) => ts,
        _ => quote!(),
    };

    let call = quote!(
        let mut __lock_state = self.0.lock().unwrap();
        let __props = ::launchpad_uri::props(&uri.path(), &self.path());
        #data

        match #name(#props) {
            Ok(__data) => ::launchpad::Response::from(__data),
            Err(__error) => ::launchpad::Response::from(::launchpad::Error::from(__error)),
        }
    );

    quote! {
         #[derive(Debug)]
         #[allow(non_camel_case_types)]
         struct #name(std::sync::Mutex<::launchpad::State<#state>>);

         #[allow(non_camel_case_types)]
         impl ::launchpad::endpoint::Endpoint for #name {
             fn methods(&self) -> Vec<hyper::Method> {
                 #methods
             }

             fn path(&self) -> String {
                 #path
             }

             fn execute(
                 &self,
                 uri: &hyper::Uri,
                 headers: &hyper::header::HeaderMap<hyper::header::HeaderValue>,
                 body: &bytes::Bytes
            ) -> ::launchpad::Response {
                 #function

                 #call
             }
         }
    }
    .into()
}
