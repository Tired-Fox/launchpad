use std::fmt::Display;

use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    braced, ext::IdentExt, parse::Parse, token::Brace, Expr, ExprMacro, ExprPath, Ident, Lit,
    LitFloat, LitStr, Token,
};

#[derive(Clone)]
pub enum FetchBody {
    None,
    Macro(ExprMacro),
    Lit(Lit),
    Ident(ExprPath),
}

#[derive(Clone)]
pub enum StrIdent {
    Str(LitStr),
    Ident(Ident),
}

impl ToTokens for StrIdent {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            StrIdent::Str(s) => s.to_tokens(tokens),
            StrIdent::Ident(i) => i.to_tokens(tokens),
        }
    }
}

impl Display for StrIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Str(s) => s.value(),
                Self::Ident(i) => i.to_string(),
            }
        )
    }
}

impl Parse for StrIdent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            Ok(StrIdent::Str(input.parse()?))
        } else if input.peek(Ident) {
            Ok(StrIdent::Ident(input.parse()?))
        } else {
            abort!(input.span(), "Expected &str or ident");
        }
    }
}

pub struct Fetch {
    uri: StrIdent,
    headers: Vec<(String, StrIdent)>,
    method: Option<String>,
    version: Option<LitFloat>,
    body: FetchBody,
}

impl Fetch {
    fn headers(&self) -> TokenStream2 {
        let mut stream = TokenStream2::new();
        if !self.headers.is_empty() {
            for (header, value) in self.headers.iter() {
                stream.append_all(quote! {
                    .header(#header, #value)
                })
            }
        }
        stream
    }

    fn method(&self) -> TokenStream2 {
        match self.method.clone() {
            Some(method) => quote! {.method(#method)},
            None => quote! {},
        }
    }

    fn version(&self) -> TokenStream2 {
        match self.version.clone() {
            Some(version) => quote! { .version(#version) },
            None => quote! {},
        }
    }

    fn body(&self) -> TokenStream2 {
        let mut stream = TokenStream2::new();
        match self.body.clone() {
            FetchBody::None => stream.append_all(quote! {
                ()
            }),
            FetchBody::Macro(expr) => expr.to_tokens(&mut stream),
            FetchBody::Lit(lit) => lit.to_tokens(&mut stream),
            FetchBody::Ident(expr) => expr.to_tokens(&mut stream),
        }
        quote!(.body(#stream))
    }
}

impl ToTokens for Fetch {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let uri = self.uri.clone();
        let headers = self.headers();
        let method = self.method();
        let version = self.version();
        let body = self.body();

        tokens.append_all(quote! {
            {
                use tela::client::SendRequest;

                ::tela::Request::builder().uri(#uri)
                #headers
                #method
                #version
                #body
                .send()
            }
        });
    }
}

const METHODS: [&'static str; 9] = [
    "get", "post", "delete", "put", "head", "connect", "options", "trace", "patch",
];

/// Convert a string of snake case to a string of Pascal Cabob
///
/// # Example
/// ```
/// {"some_value": "value"}
/// ```
///
/// to
///
/// ```
/// {"Some-Value": "value"}
/// ```
fn snake_to_cabob(s: String) -> String {
    s.split("_")
        .map(|s| {
            if s.len() > 0 {
                s.chars()
                    .enumerate()
                    .map(|(idx, c)| {
                        if idx == 0 {
                            c.to_ascii_uppercase()
                        } else {
                            c.to_ascii_lowercase()
                        }
                    })
                    .collect::<String>()
            } else {
                s.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("-")
}

impl Fetch {
    fn parse_method(input: syn::parse::ParseStream) -> syn::Result<String> {
        let method = input.parse::<Ident>()?;
        if METHODS.contains(&method.to_string().to_lowercase().as_str()) {
            Ok(method.to_string().to_uppercase())
        } else {
            abort!(method.span(), "Unkown request method"; help="Try `get`, `post`, `delete`, `put`, `head`, `connect`, `options`, `trace`, `patch`");
        }
    }

    fn parse_version(input: syn::parse::ParseStream) -> syn::Result<LitFloat> {
        let http = input.parse::<Ident>()?;
        if http.to_string().to_lowercase().as_str() != "http" {
            abort!(input.span(), "Expected HTTP identifier"; help="Try `http`");
        }
        let _ = input.parse::<Token![/]>()?;
        Ok(input.parse::<LitFloat>()?)
    }

    fn parse_headers(input: syn::parse::ParseStream) -> syn::Result<Vec<(String, StrIdent)>> {
        let mut headers = Vec::new();
        while !input.is_empty() {
            let key = StrIdent::parse(input)?;

            if input.peek(Token![,]) {
                let temp = key.clone();
                match key {
                    StrIdent::Ident(ident) => {
                        headers.push((snake_to_cabob(ident.to_string()), temp))
                    }
                    _ => abort!(
                        input.span(),
                        "Can only use shorthand header assignment with identifiers"
                    ),
                };
            } else if input.peek(Token![:]) {
                let _ = input.parse::<Token![:]>();
                headers.push((snake_to_cabob(key.to_string()), StrIdent::parse(input)?));
            }

            if !input.peek(Token![,]) && !input.is_empty() {
                abort!(input.span(), "Invalid header assignment syntax");
            }
            let _ = input.parse::<Token![,]>();
        }
        Ok(Vec::new())
    }

    fn parse_body(input: syn::parse::ParseStream) -> syn::Result<FetchBody> {
        let expr = input.parse::<Expr>()?;
        match expr {
            Expr::Lit(val) => {
                println!("LITERAL");
                Ok(FetchBody::Lit(val.lit))
            }
            Expr::Macro(val) => {
                println!("MACRO");
                Ok(FetchBody::Macro(val))
            }
            Expr::Path(path) => Ok(FetchBody::Ident(path)),
            _ => abort!(
                input.span(),
                "Invalid fetch body syntax";
                help = "Only literals, macros, and identifiers are supported"
            ),
        }
    }
}

impl Parse for Fetch {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let uri = if input.peek(LitStr) {
            StrIdent::Str(input.parse::<LitStr>()?)
        } else {
            StrIdent::Ident(input.parse::<Ident>()?)
        };

        let mut fetch = Fetch {
            uri,
            headers: Vec::new(),
            method: None,
            version: None,
            body: FetchBody::None,
        };

        while !input.is_empty() {
            if input.peek(Ident::peek_any) {
                let key = input.parse::<Ident>()?;

                let _ = input.parse::<Token![:]>()?;
                match key.to_string().as_str() {
                    "method" => fetch.method = Some(Fetch::parse_method(input)?),
                    "version" => fetch.version = Some(Fetch::parse_version(input)?),
                    "body" => fetch.body = Fetch::parse_body(input)?,
                    _ => {
                        abort!(key.span(), "Unkown fetch option"; help="Try `method`, `version`, or `body`")
                    }
                }
            } else if input.peek(Brace) {
                let headers;
                braced!(headers in input);
                fetch.headers = Fetch::parse_headers(&headers)?;
            } else if input.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                abort!(input.span(), "Invalid fetch option syntax");
            }
        }

        Ok(fetch)
    }
}
