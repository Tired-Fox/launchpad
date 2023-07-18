use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, GenericArgument, ItemFn, Pat, PatType, PathArguments, Type, Visibility};

#[derive(Default)]
pub struct PresentProps {
    pub state: Option<TokenStream>,
    pub data: Option<TokenStream>,
    pub query: Option<TokenStream>,
}

pub enum Identifier {
    State(bool, Type),
    Data(Type),
    Query(Type),
    Prop(String),
}

fn identify(prop: (String, Type)) -> Identifier {
    if let Some((mutable, inner_type)) = get_state(&prop.1) {
        return Identifier::State(mutable, inner_type);
    }

    if let Some(inner_type) = get_data(&prop.1) {
        return Identifier::Data(inner_type);
    }

    if let Some(inner_type) = get_query(&prop.1) {
        return Identifier::Query(inner_type);
    }

    Identifier::Prop(prop.0)
}

fn get_state(prop: &Type) -> Option<(bool, Type)> {
    match prop {
        Type::Reference(r) => {
            if let Type::Path(path) = &*r.elem {
                if let Some(seg) = path.path.segments.last() {
                    if seg.ident.to_string() == "State".to_string() {
                        let elem = match &seg.arguments {
                            PathArguments::AngleBracketed(brackets) => {
                                if brackets.args.len() == 1 {
                                    match &brackets.args[0] {
                                        GenericArgument::Type(t) => t.clone(),
                                        _ => panic!("Expected State<T> generic type to be a type"),
                                    }
                                } else {
                                    panic!("Expected one State<T> generic type")
                                }
                            }
                            _ => panic!("Expected State<T> generic type"),
                        };

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
        Type::Path(p) => {
            if let Some(seg) = p.path.segments.last() {
                if seg.ident.to_string() == "State".to_string() {
                    panic!("Expected 'State<_>' argument to be a reference: '&State<_>' or '&mut State<_>'")
                }
            }
        }
        _ => return None,
    };
    None
}

fn get_data(prop: &Type) -> Option<Type> {
    match prop {
        Type::Reference(r) => {
            if let Type::Path(path) = &*r.elem {
                if let Some(seg) = path.path.segments.last() {
                    if seg.ident.to_string() == "Data".to_string() {
                        panic!("Expected 'Data<_>' argument to move the variable: was refernce, but should move")
                    }
                }
            }
        }
        Type::Path(p) => {
            if let Some(seg) = p.path.segments.last() {
                if seg.ident.to_string() == "Data".to_string() {
                    match &seg.arguments {
                        PathArguments::AngleBracketed(brackets) => {
                            if brackets.args.len() == 1 {
                                match &brackets.args[0] {
                                    GenericArgument::Type(t) => return Some(t.clone()),
                                    _ => panic!("Expected Data<T> generic type to be a type"),
                                }
                            } else {
                                panic!("Expected one Data<T> generic type")
                            }
                        }
                        _ => panic!("Expected Data<T> generic type"),
                    };
                }
            }
        }
        _ => return None,
    };
    None
}

fn get_query(prop: &Type) -> Option<Type> {
    match prop {
        Type::Reference(r) => {
            if let Type::Path(path) = &*r.elem {
                if let Some(seg) = path.path.segments.last() {
                    if seg.ident.to_string() == "Query".to_string() {
                        panic!("Expected 'Query<_>' argument to move the variable: was refernce, but should move")
                    }
                }
            }
        }
        Type::Path(p) => {
            if let Some(seg) = p.path.segments.last() {
                if seg.ident.to_string() == "Query".to_string() {
                    match &seg.arguments {
                        PathArguments::AngleBracketed(brackets) => {
                            if brackets.args.len() == 1 {
                                match &brackets.args[0] {
                                    GenericArgument::Type(t) => return Some(t.clone()),
                                    _ => panic!("Expected Query<T> generic type to be a type"),
                                }
                            } else {
                                panic!("Expected one Query<T> generic type")
                            }
                        }
                        _ => panic!("Expected Query<T> generic type"),
                    };
                }
            }
        }
        _ => return None,
    };
    None
}

/// Parse the function arguments and return a vector of types
fn parse_props(function: &ItemFn) -> Vec<(String, Type)> {
    function
        .sig
        .inputs
        .clone()
        .into_iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(PatType { ty, pat, .. }) => {
                let name = match &*pat {
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

pub fn compile_props(function: &ItemFn, include_data: &bool) -> (PresentProps, TokenStream) {
    let mut results = Vec::new();
    let mut present = PresentProps::default();

    for prop in parse_props(&function) {
        match identify(prop) {
            Identifier::State(mutable, inner_type) => {
                match present.state {
                    Some(_) => panic!("More than one 'State<_>' parameter in function"),
                    _ => {
                        match mutable {
                            true => results.push("&mut *__lock_state".to_string()),
                            _ => results.push("&*__lock_state".to_string()),
                        }
                        present.state = Some(quote!(#inner_type));
                    }
                };
            }
            Identifier::Prop(name) => {
                results.push(format!("__props.remove(\"{}\").unwrap().into()", name));
            }
            Identifier::Query(inner_type) => match present.query {
                Some(_) => panic!("More than one 'Query<_>' parameter in function"),
                _ => {
                    results.push("__query".to_string());
                    present.query = Some(quote! {
                        let __query = match ::launchpad::request::Query::<#inner_type>::parse(uri) {
                            Ok(__q) => __q,
                            Err(e) => return ::launchpad::Response::from(e)
                        };
                    });
                }
            }
            Identifier::Data(inner_type) => match present.data {
                Some(_) => panic!("More than one 'Data<_>' parameter in function"),
                _ => {
                    if !*include_data {
                        panic!("Request method cannot parse a request body (Data<_>)")
                    }

                    results.push("__data".to_string());
                    present.data = Some(quote! {
                        let __data = match ::launchpad::request::Data::<#inner_type>::parse(headers, body) {
                            Ok(__d) => __d,
                            Err(e) => return ::launchpad::Response::from(e)
                        };
                    });
                }
            },
        }
    }

    (
        present,
        results.join(", ").parse::<TokenStream>().unwrap().into(),
    )
}
