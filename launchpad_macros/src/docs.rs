/// Compile functon docs into a single TokenStream2 docs.
/// uses the `///` format for each line
pub(crate) fn compile_docs(function: &mut syn::ItemFn) -> String {
    // Collect doc attributes from function and remove them from the function
    let mut docs = Vec::new();
    function.attrs = function.attrs.iter().filter(|attr| {
        match &attr.meta {
            syn::Meta::NameValue(syn::MetaNameValue{path: syn::Path{segments, ..}, eq_token: _, value}) => {
                if segments.last().unwrap().ident.to_string() == "doc" {
                    match &value {
                        syn::Expr::Lit(syn::ExprLit{lit, ..}) => {
                            match &lit {
                                syn::Lit::Str(val) => {
                                    docs.push(val.value());
                                    return false;
                                },
                                //Unkown lit
                                _ => {}
                            }
                        },
                        //unkown meta name value
                        _ => {}
                    }
                }
            },
            //Unkown meta type 
            _ => {}
        };
        true
    }).map(|a| a.clone()).collect::<Vec<syn::Attribute>>();

    // Construct struct docs from method doc attributes
    docs
        .iter()
        .map(|d| {
            if d.contains('\n') {
                d.trim().split("\n").map(|p| match p.trim().strip_prefix("*") {
                    Some(val) => val.to_string(),
                    None => p.to_string()
                }).collect::<Vec<String>>().join("\n")

            } else {
                d.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}
