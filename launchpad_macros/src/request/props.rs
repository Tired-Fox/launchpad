use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{FnArg, GenericArgument, ItemFn, Pat, PatType, PathArguments, Type};

#[derive(Default)]
pub struct PresentProps {
    pub state: Option<TokenStream>,
    pub content: Option<TokenStream>,
    pub query: Option<TokenStream>,
}

pub enum Identifier {
    State(bool, Type),
    Content(Type),
    Query(Type),
    Prop(String),
}

fn identify(prop: &(String, Type)) -> Identifier {
    if let Some((mutable, inner_type)) = get_request_param(&prop.1, "State", true) {
        return Identifier::State(mutable, inner_type);
    }

    if let Some((_, inner_type)) = get_request_param(&prop.1, "Content", false) {
        return Identifier::Content(inner_type);
    }

    if let Some((_, inner_type)) = get_request_param(&prop.1, "Query", false) {
        return Identifier::Query(inner_type);
    }

    Identifier::Prop(prop.0.clone())
}

/// Check for valid segments in the type path.
///
/// This is limited and could fail. There is no way of determining the
/// full type path in the macro.
///
/// launchpad::request::<type>
/// request::<type>
/// <type>
fn in_request_module(segments: Vec<String>) -> bool {
    if segments.len() > 1 {
        let path = vec!["launchpad".to_string(), "request".to_string()];
        let valid = &path[0+segments.len()-1..];
        for i in 0..valid.len() {
            if valid[i] != segments[i] {
                return false;
            }
        }
    }
    true
}

fn get_request_param(prop: &Type, name: &str, referenced: bool) -> Option<(bool, Type)> {
    let (mutable, path) = match referenced {
        true => {
            if let Type::Reference(r) = prop {
                (r.mutability.is_some(), match &*r.elem {
                    Type::Path(path) => path,
                    _ => return None
                })
            } else {
                if let Type::Path(path) = prop {
                    if let Some(seg) = path.path.segments.last() {
                        if seg.ident.to_string() == name.to_string() {
                            abort!(prop, "Expected reference or mut reference")
                        }
                    }
                }
                return None;
            }
        },
        false => {
            if let Type::Reference(r) = prop {
                if let Type::Path(path) = &*r.elem {
                    if let Some(seg) = path.path.segments.last() {
                        if seg.ident.to_string() == name.to_string() {
                            abort!(prop, "Expected ownership of parameter, is reference")
                        }
                    }
                }
                return None;
            } else if let Type::Path(path) = prop {
                (false, path)
            } else {
                return None
            }
        }
    };

    let segments = path.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<String>>();
    if let Some(seg) = path.path.segments.last() {
        if !in_request_module(segments) {
            // Not a launchpad State
            return None
        }

        if seg.ident.to_string() == name.to_string() {
            let elem = match &seg.arguments {
                PathArguments::AngleBracketed(brackets) => {
                    if brackets.args.len() == 1 {
                        match &brackets.args[0] {
                            GenericArgument::Type(t) => t.clone(),
                            _ => abort!(prop, "Expected generic type to be a type"),
                        }
                    } else {
                        abort!(prop, "Expected one generic type")
                    }
                }
                _ => abort!(prop, "Expected generic type"),
            };

            return Some((
                mutable,
                elem,
            ));
        }
    }
    None
}

// fn get_state(prop: &Type) -> Option<(bool, Type)> {
//     match prop {
//         Type::Reference(r) => {
//             if let Type::Path(path) = &*r.elem {
//                 let segments = path.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<String>>();
//                 if let Some(seg) = path.path.segments.last() {
//                     if !in_request_module(segments) {
//                         // Not a launchpad State
//                         return None
//                     }
//
//                     if seg.ident.to_string() == "State".to_string() {
//                         let elem = match &seg.arguments {
//                             PathArguments::AngleBracketed(brackets) => {
//                                 if brackets.args.len() == 1 {
//                                     match &brackets.args[0] {
//                                         GenericArgument::Type(t) => t.clone(),
//                                         _ => abort!(prop, "Expected generic type to be a type"),
//                                     }
//                                 } else {
//                                     abort!(prop, "Expected one generic type")
//                                 }
//                             }
//                             _ => abort!(prop, "Expected generic type"),
//                         };
//
//                         return Some((
//                             match r.mutability {
//                                 Some(_) => true,
//                                 None => false,
//                             },
//                             elem,
//                         ));
//                     }
//                 }
//             }
//         }
//         Type::Path(p) => {
//             if let Some(seg) = p.path.segments.last() {
//                 if seg.ident.to_string() == "State".to_string() {
//                     abort!(prop, "Expected reference or mut reference")
//                 }
//             }
//         }
//         _ => return None,
//     };
//     None
// }
//
// fn get_content(prop: &Type) -> Option<Type> {
//     match prop {
//         Type::Reference(r) => {
//             if let Type::Path(path) = &*r.elem {
//                 if let Some(seg) = path.path.segments.last() {
//                     if seg.ident.to_string() == "Content".to_string() {
//                         abort!(prop, "Expected to move parameter, but was referenced")
//                     }
//                 }
//             }
//         }
//         Type::Path(p) => {
//             if let Some(seg) = p.path.segments.last() {
//                 if seg.ident.to_string() == "Content".to_string() {
//                     match &seg.arguments {
//                         PathArguments::AngleBracketed(brackets) => {
//                             if brackets.args.len() == 1 {
//                                 match &brackets.args[0] {
//                                     GenericArgument::Type(t) => return Some(t.clone()),
//                                     _ => abort!(prop, "Expected generic type to be a type"),
//                                 }
//                             } else {
//                                 abort!(prop, "Expected one generic type")
//                             }
//                         }
//                         _ => abort!(prop, "Expected generic type"),
//                     };
//                 }
//             }
//         }
//         _ => return None,
//     };
//     None
// }
//
// fn get_query(prop: &Type) -> Option<Type> {
//     match prop {
//         Type::Reference(r) => {
//             if let Type::Path(path) = &*r.elem {
//                 if let Some(seg) = path.path.segments.last() {
//                     if seg.ident.to_string() == "Query".to_string() {
//                         abort!(prop, "Expected to move parameter, but was referenced");
//                     }
//                 }
//             }
//         }
//         Type::Path(p) => {
//             if let Some(seg) = p.path.segments.last() {
//                 if seg.ident.to_string() == "Query".to_string() {
//                     match &seg.arguments {
//                         PathArguments::AngleBracketed(brackets) => {
//                             if brackets.args.len() == 1 {
//                                 match &brackets.args[0] {
//                                     GenericArgument::Type(t) => return Some(t.clone()),
//                                     _ => abort!(prop, "Expected generic type to be a type"),
//                                 }
//                             } else {
//                                 abort!(prop, "Expected one generic type")
//                             }
//                         }
//                         _ => abort!(prop, "Expected generic type"),
//                     };
//                 }
//             }
//         }
//         _ => return None,
//     };
//     None
// }

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
                        abort!(ty, "Expected named argument")
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

    for prop in parse_props(&function).iter() {
        match identify(prop) {
            Identifier::State(mutable, inner_type) => {
                match present.state {
                    Some(_) => abort!(prop.1, "More than one 'State<_>' parameter in function"),
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
                Some(_) => abort!(prop.1, "More than one 'Query<_>' parameter in function"),
                _ => {
                    results.push("__query".to_string());
                    present.query = Some(quote! {
                        let __query = match ::launchpad::request::Query::<#inner_type>::parse(uri) {
                            Ok(__q) => __q,
                            Err(e) => return ::launchpad::Response::from(e)
                        };
                    });
                }
            },
            Identifier::Content(inner_type) => match present.content {
                Some(_) => abort!(prop.1, "More than one 'Content<_>' parameter in function"),
                _ => {
                    if !*include_data {
                        abort!(
                            prop.1,
                            "Request method cannot parse a request body (Content<_>)"
                        )
                    }

                    results.push("__content".to_string());
                    present.content = Some(quote! {
                        let __content = match ::launchpad::request::Content::<#inner_type>::parse(headers, body) {
                            Ok(__c) => __c,
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
