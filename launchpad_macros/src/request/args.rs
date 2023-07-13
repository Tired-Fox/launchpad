use syn::{parse::Parse, Ident, LitStr, Result};

pub struct Args {
    pub path: Option<LitStr>,
    pub methods: Vec<String>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            path: None,
            methods: vec!["GET".to_string()],
        }
    }
}

impl Parse for Args {
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

        Ok(Args { path, methods })
    }
}
