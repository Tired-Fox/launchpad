use proc_macro_error::abort;
use syn::{parse::Parse, LitInt};

pub(crate) struct CatchArgs {
    pub code: syn::LitInt,
}

impl Parse for CatchArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let code = match input.parse::<LitInt>() {
            Ok(c) => c,
            _ => abort!(input.span(), "Must provide only a single u16 error code")
        };

        Ok(CatchArgs { code })
    }
}
