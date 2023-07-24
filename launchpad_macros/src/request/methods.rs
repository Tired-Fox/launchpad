use proc_macro2::TokenStream;

use super::RequestArgs;

pub(crate) fn compile_methods_vec(args: &RequestArgs) -> TokenStream {
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

pub(crate) fn build_method_comment_list(args: &RequestArgs) -> String {
    let methods = args.methods.iter().map(|m| format!("`{}`", m)).collect::<Vec<String>>();
    if args.methods.len() > 2 {
        let mut result = (&methods[0..methods.len()-1]).join(", ");
        result.push_str(format!("and {}", methods.last().unwrap()).as_str());
        result
    } else if args.methods.len() == 2 {
        format!("{} and {}", args.methods[0], args.methods[1])
    } else {
        args.methods[0].clone()
    }
}
