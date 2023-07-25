use proc_macro2::Span;
use proc_macro_error::abort;
use syn::{bracketed, parse::Parse, punctuated::Punctuated, Ident, LitInt, LitStr, Result, Token};

pub struct RequestArgs {
    pub path: Option<LitStr>,
    pub methods: Vec<String>,
}

impl Default for RequestArgs {
    fn default() -> Self {
        Self {
            path: None,
            methods: vec!["GET".to_string()],
        }
    }
}

impl Parse for RequestArgs {
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

        Ok(RequestArgs { path, methods })
    }
}

pub(crate) struct CatchArgs {
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
