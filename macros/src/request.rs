use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::abort;
use quote::quote;
use syn::{
    bracketed, parse::Parse, punctuated::Punctuated, FnArg, Ident, ItemFn, LitInt, LitStr, Pat,
    PatIdent, PatType, Result, Token, Visibility,
};

use super::{
    docs::compile_docs,
    helpers::{get_path_generic, get_path_name},
};

pub struct RequestArgs {
    pub path: LitStr,
    pub methods: Vec<String>,
}

impl Parse for RequestArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path = input
            .parse::<LitStr>()
            .map_err(|_| abort!(input.span(), "Expected uri path"))
            .unwrap();
        let _: Result<Token![,]> = input.parse();

        let mut methods = Vec::new();
        if input.peek(Ident) {
            let next: Ident = input.parse()?;
            if next != "methods" {
                abort!(input.span(), "Unkown argument");
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

        Ok(RequestArgs { path, methods })
    }
}

pub struct CatchArgs {
    pub code: syn::LitInt,
}

impl Parse for CatchArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(CatchArgs {
                code: LitInt::new("0", Span::call_site()),
            });
        }

        let code = match input.parse::<LitInt>() {
            Ok(c) => c,
            _ => match input.parse::<Ident>() {
                Ok(c) if c.to_string().as_str() == "all" => LitInt::new("0", Span::call_site()),
                _ => abort!(
                    input.span(),
                    "Expected single u16 or `all` identifier argument"
                ),
            },
        };

        Ok(CatchArgs { code })
    }
}

fn parse_props(path: String, function: &ItemFn) -> TokenStream2 {
    let mut props: Vec<String> = Vec::new();
    let captures: Vec<String> = path
        .split("/")
        .filter_map(|p| {
            if p.starts_with(":...") {
                Some(p.strip_prefix(":...").unwrap().to_string())
            } else if p.starts_with(":") {
                Some(p.strip_prefix(":").unwrap().to_string())
            } else {
                None
            }
        })
        .collect();

    let error = |a: FnArg| {
        abort!(
            a,
            format!("Invalid endpoint argument: expected Query or Body types; or a uri capture")
        )
    };

    for arg in function.sig.inputs.iter() {
        match arg {
            FnArg::Typed(PatType { ty, pat, .. }) => {
                match get_path_name(ty).as_str() {
                    "Option" => match get_path_name(&get_path_generic(ty)).as_str() {
                        "Body" => props.push(format!(
                            "::wayfinder::request::Body::extract(body.to_owned()).ok()"
                        )),
                        "Query" => {
                            props.push(format!("::wayfinder::request::Query::extract(uri).ok()"))
                        }
                        _ => match &(**pat) {
                            Pat::Ident(PatIdent { ident, .. })
                                if captures.contains(&ident.to_string()) =>
                            {
                                props.push(format!(
                                    "__captures.get(\"{}\").unwrap_or(\"\").parse::<{}>().ok()",
                                    ident,
                                    quote!(#ty)
                                ))
                            }
                            ty => abort!(
                                arg,
                                format!("Expected a uri capture, was {:?}: {:?}", ty, captures)
                            ),
                        },
                    },
                    "Body" => props.push(format!(
                        "::wayfinder::request::Body::extract(body.to_owned())?"
                    )),
                    "Query" => props.push(format!("::wayfinder::request::Query::extract(uri)?")),
                    _ => match &(**pat) {
                        Pat::Ident(PatIdent { ident, .. })
                            if captures.contains(&ident.to_string()) =>
                        {
                            props.push(format!(
                                "__captures.get(\"{}\").unwrap().parse::<{}>().unwrap()",
                                ident,
                                quote!(#ty)
                            ))
                        }
                        ty => abort!(
                            arg,
                            format!("Expected a uri capture, was {:?}: {:?}", ty, captures)
                        ),
                    },
                };
            }
            _ => error(arg.clone()),
        }
    }
    props.join(",").parse::<TokenStream2>().unwrap()
}

pub fn request_endpoint(args: RequestArgs, mut function: ItemFn) -> TokenStream {
    let uri = args.path.value();
    let path = args.path;

    let docs = format!(
        "#[doc=\"Request endpoint for URIs matching `{}`\n\n{}\"]",
        uri,
        compile_docs(&mut function)
    )
    .parse::<TokenStream2>()
    .unwrap();

    let methods = format!(
        "vec![{}]",
        args.methods
            .iter()
            .map(|m| format!("hyper::Method::{}", m.to_uppercase()))
            .collect::<Vec<String>>()
            .join(",")
    )
    .parse::<TokenStream2>()
    .unwrap();

    let props = parse_props(path.value().to_string(), &function);
    let name = function.sig.ident.clone();
    let vis = function.vis.clone();
    function.sig.ident = Ident::new("__call", function.sig.ident.span());
    function.vis = Visibility::Inherited;

    quote! {
        #docs
        #[allow(non_camel_case_types)]
        #[derive(Debug)]
        #vis struct #name;
        impl ::wayfinder::request::Endpoint for #name {
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
                uri: &mut hyper::Uri,
                body: &mut Vec<u8>,
            ) -> ::wayfinder::response::Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
                #[inline]
                #function

                let __captures = ::wayfinder::uri::props(&uri.path().to_string(), &self.path());
                __call(#props).to_response()
            }
        }
    }
    .into()
}

pub fn request_catch(args: CatchArgs, mut function: ItemFn) -> TokenStream {
    let name = function.sig.ident.clone();
    let vis = function.vis.clone();
    let code = args.code;
    let docs = format!(
        "#[doc=\"Catches {} errors and handles them\n\n{}\"]",
        match code.to_string().as_str() {
            "0" => "any",
            val => val,
        },
        compile_docs(&mut function)
    )
    .parse::<TokenStream2>()
    .unwrap();

    function.sig.ident = proc_macro2::Ident::new("__callback", function.sig.ident.span());
    function.vis = syn::Visibility::Inherited;

    quote! {
        #docs
        #[derive(Debug)]
        #[allow(non_camel_case_types)]
        #vis struct #name;

        #[allow(non_camel_case_types)]
        impl ::wayfinder::request::Catch for #name {
            fn execute(
                &self,
                code: u16,
                message: String,
                reason: String
            ) -> ::wayfinder::response::Result<hyper::Response<http_body_util::Full<bytes::Bytes>>> {
                #function

                __callback(code.clone(), message, reason.clone()).to_error_response(code, reason)
            }

            #[inline]
            fn code(&self) -> u16 {
                #code
            }
        }
    }
    .into()
}
