use proc_macro_error::abort;
use syn::{parse::{Parse, ParseStream}, Token, braced};
mod args;

pub(crate) use args::CatchArgs;

pub(crate) struct Routes {
    pub endpoints: Vec<String>,
    pub catches: Vec<String>,
}

/// rts!{
///     index,
///     "/api/data/<username>" => data,
///     ERRORS {
///         404 => not_found,
///         internal_server
///     }
/// }
impl Parse for Routes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut routes = Routes {
            endpoints: Vec::new(),
            catches: Vec::new()
        };

        while !input.is_empty() {
            match input.parse::<syn::Ident>() {
                Ok(ident) => {
                    match ident.to_string().to_lowercase().as_str() {
                        "errors" => {
                            match input.parse::<Token![:]>() {
                                Ok(_) => {
                                    let data;
                                    braced!(data in input);
                                    parse_catches(&&data, &mut routes);
                                },
                                _ => {
                                    abort!(input.span(), "Expected `: {...}` mapping after ERRORS identifier")
                                }
                            }
                        },
                        "routes" => {
                            match input.parse::<Token![:]>() {
                                Ok(_) => {
                                    let data;
                                    braced!(data in input);
                                    parse_endpoints(&&data, &mut routes);
                                },
                                _ => {
                                    abort!(input.span(), "Expected `: {...}` mapping after ROUTES identifier")
                                }
                            }
                        },
                        _ => abort!(input.span(), "Expected `ROUTES` or `ERRORS` identifier")

                    }
                },
                Err(_) => abort!(input.span(), "Expected `ROUTES` or `ERRORS` identifier")
            }
            let _ = input.parse::<Token![,]>();
        }

        Ok(routes)
    }
}

fn parse_endpoints(input: syn::parse::ParseStream, routes: &mut Routes) {
    while !input.is_empty() {
        match input.parse::<syn::Ident>() {
            Ok(ident) => {
                routes.endpoints.push(format!(
                    "::launchpad::router::Route::from_endpoint(std::sync::Arc::new({}(std::sync::Mutex::new(::launchpad::request::State::default()))))",
                    ident
                ))
            },
            _ => match input.parse::<syn::LitStr>() {
                Ok(lit) => {
                    let _ = input.parse::<Token![=>]>().map_err(|_| abort!(input.span(), "Expected `=>` mapping from path to endpoint"));
                    let value = input
                        .parse::<syn::Ident>()
                        .map_err(|_| abort!(input.span(), "Expected identifier after `=>`"))
                        .unwrap();
                    routes.endpoints.push(format!(
                        "::launchpad::router::Route::new(
                            \"{}\".to_string(),
                            std::sync::Arc::new({}(std::sync::Mutex::new(::launchpad::request::State::default())))
                        )",
                        lit.value(),
                        value
                    ))
                },
                _ => abort!(input.span(), "Expected identifier or string literal")
            }
        }
        let _ = input.parse::<Token![,]>();
    }
}

fn parse_catches(input: ParseStream, routes: &mut Routes) {
    while !input.is_empty() {
        match input.parse::<syn::Ident>() {
            Ok(handler) => {
                routes.catches.push(format!("::launchpad::router::Catch::from_catch(std::sync::Arc::new({}()))", handler))
            },
            _ => match input.parse::<syn::LitInt>() {
                Ok(code) => {
                    let _ = input.parse::<Token![=>]>().map_err(|_| abort!(input.span(), "Expected `=>` mapping after u16 code"));
                    let handler = input
                        .parse::<syn::Ident>()
                        .map_err(|_| abort!(input.span(), "Expected identifier after `=>` mapping"))
                        .unwrap();
                    routes.catches.push(format!("::launchpad::router::Catch::new({}, std::sync::Arc::new({}()))", code, handler))
                },
                _ => abort!(input.span(), "Expected identifier or u16 number")
            }
        }
        let _ = input.parse::<Token![,]>();
    }
}
