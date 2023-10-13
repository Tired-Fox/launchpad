use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_error::{abort, emit_error};
use quote::{quote, ToTokens, TokenStreamExt};
use std::{collections::HashMap, fmt::Debug};
use syn::{
    braced,
    ext::IdentExt,
    parse::{Parse, ParseBuffer, ParseStream},
    token::Brace,
    Ident, LitStr, Token,
};

mod constants;
use constants::TAGS;

pub enum AET {
    Push,
    Extend,
    OptionalExtend,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum TokenType {
    #[default]
    None,
    Element,
    Comment,
    Text,
    Capture,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Token {
    pub ttype: TokenType,
    pub payload: usize,
}

#[derive(Default, Clone, Debug)]
pub enum Attribute {
    #[default]
    Exists,
    Literal(usize),
    Capture(usize),
}

#[derive(Debug, Clone)]
pub struct Spanned<T: Debug + Clone> {
    span: Span,
    value: T,
}

impl<T: Default + Clone + Debug + ToTokens> ToTokens for Spanned<T> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.value().to_tokens(tokens)
    }
}

impl<T: Default + Clone + Debug> Default for Spanned<T> {
    fn default() -> Self {
        Spanned {
            span: Span::call_site(),
            value: T::default(),
        }
    }
}

impl<T: Debug + Clone + Default> Spanned<T> {
    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn inner(&self) -> T {
        self.value.clone()
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Default, Clone, Debug)]
pub struct Element {
    pub tag: usize,
    pub attrs: Option<usize>,
    pub spread: Option<usize>,
    pub captures: Vec<usize>,
    pub children: Option<usize>,
    pub decl: bool,
}

#[derive(Debug, Default, Clone)]
pub struct Parser {
    pub root: Vec<usize>,
    pub stack: Vec<usize>,
    pub tokens: Vec<Token>,

    pub elements: Vec<Element>,
    pub children: Vec<Vec<usize>>,
    pub spreads: Vec<Spanned<TokenStream2>>,

    pub tags: Vec<Spanned<String>>,
    tag_map: HashMap<String, usize>,

    pub captures: Vec<Spanned<TokenStream2>>,
    pub attrs: Vec<HashMap<String, Spanned<Attribute>>>,
    pub content: Vec<Spanned<String>>,
}

impl ToTokens for Parser {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let children: Vec<(AET, TokenStream2)> = self
            .root
            .iter()
            .filter_map(|t| self.tokenize(&self.tokens[*t]))
            .collect();

        if children.len() > 0 {
            let chldrn = self.tokenize_children(children);
            tokens.append_all(quote!({
                use tela::html::prelude::*;

                Element::wrapper(#chldrn)
            }))
        }
    }
}

impl Parse for Parser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parser = Parser::new();
        parser.parse(input)?;
        Ok(parser)
    }
}

macro_rules! token {
    ($ttype: ident, $payload: expr) => {
        Token {
            ttype: TokenType::$ttype,
            payload: $payload,
        }
    };
}

macro_rules! spanned {
    ($input: ident, @text) => {{
        let lit_str = $input.parse::<syn::LitStr>()?;
        Spanned {
            span: lit_str.span(),
            value: lit_str.value(),
        }
    }};
    ($input: ident, @capture) => {
        {
            let braces;
            syn::braced!(braces in $input);
            Spanned { span: braces.span(), value: braces.parse::<TokenStream2>()? }
        }
    };
}

/// Eat tokens that fit in Token![...] from the input
macro_rules! eat {
    ($input: ident, $($token: tt)*) => {
        {
            $(let _ = $input.parse::<Token![$token]>();)*
        }
    };
}

macro_rules! on_peek {
    ($input: ident { $first: expr => {$($body: tt)*} }) => {
        if $input.peek($first) {
            $($body)*
        }
    };
    ($input: ident { $first: expr => {$($first_body: tt)*}; $($next: expr => {$($body: tt)*});* $(;)?}) => {
        if $input.peek($first) {
            $($first_body)*
        }
        $(
            else if $input.peek($next) {
                $($body)*
            }
        )*
    };
    ($input: ident { $first: expr => {$($body: tt)*}; else => { $($else: tt)* } $(;)? }) => {
        if $input.peek($first) {
            $($body)*
        }
    };
    ($input: ident { $first: expr => {$($first_body: tt)*}; $($next: expr => {$($body: tt)*});*; else => { $($else: tt)* } $(;)? }) => {
        if $input.peek($first) {
            $($first_body)*
        }
        $(
            else if $input.peek($next) {
                $($body)*
            }
        )*
    };
}

impl Parser {
    pub fn new() -> Self {
        Parser::default()
    }

    fn parse_name(&self, input: ParseStream) -> syn::Result<Spanned<String>> {
        let first = Ident::parse_any(input)?;
        let mut name = first.to_string();
        loop {
            if input.peek(Token![-]) {
                eat!(input, -);
                name.push('-');
            } else if input.peek(Token![:]) {
                eat!(input, :);
                name.push(':');
            } else {
                break;
            }
            name.push_str(Ident::parse_any(input)?.to_string().as_str());
        }

        Ok(Spanned {
            span: first.span(),
            value: name,
        })
    }

    fn parse_attributes(
        &mut self,
        input: ParseStream,
    ) -> syn::Result<(Option<usize>, Option<usize>, Vec<usize>)> {
        // ident or {}
        // (="" | ={})?
        let mut attrs: HashMap<String, Spanned<Attribute>> = HashMap::new();

        let mut captures: Vec<usize> = Vec::new();
        let mut spread: Option<usize> = None;

        while !input.peek(Token![/]) && !input.peek(Token![>]) {
            let lookahead = input.lookahead1();
            on_peek!(lookahead {
                Ident::peek_any => {
                    let name = self.parse_name(input)?;
                    if input.peek(Token![=]) {
                        eat!(input, =);
                        let lookahead = input.lookahead1();
                        on_peek!(lookahead {
                            LitStr => {
                                let temp = spanned!(input, @text);
                                attrs.insert(name.inner(), Spanned { span: temp.span(), value: Attribute::Literal(self.content.len()) });
                                self.content.push(temp);
                            };
                            Brace => {
                                let temp = spanned!(input, @capture);
                                attrs.insert(name.inner(), Spanned { span: temp.span(), value: Attribute::Capture(self.captures.len()) });
                                self.captures.push(temp);
                            };
                            else => {
                                let err = lookahead.error();
                                abort!(err.span(), err.to_string())
                            };
                        });
                    } else {
                        attrs.insert(name.inner(), Spanned { span: name.span(), value: Attribute::Exists });
                    }
                };
                Brace => {
                    let braces: ParseBuffer;
                    braced!(braces in input);

                    if braces.peek(Token![..]) {
                        eat!(braces, ..);
                        spread = Some(self.spreads.len());
                        self.spreads.push(Spanned { span: braces.span(), value: braces.parse::<TokenStream2>()? });
                    } else {
                        captures.push(self.captures.len());
                        self.captures.push(Spanned { span: braces.span(), value: braces.parse::<TokenStream2>()? });
                    }
                };
                else => {
                    let err = lookahead.error();
                    abort!(err.span(), err.to_string())
                }
            })
        }

        let attributes = if attrs.len() > 0 {
            self.attrs.push(attrs);
            Some(self.attrs.len() - 1)
        } else {
            None
        };
        Ok((attributes, spread, captures))
    }

    fn get_tag(&self, token: usize) -> Option<Spanned<String>> {
        let token = self.tokens[token];
        if let TokenType::Element = token.ttype {
            return Some(self.tags[self.elements[token.payload].tag].clone());
        }
        None
    }

    fn add_child(&mut self, index: usize) {
        if self.stack.len() > 0 {
            let element = &self.elements[self.tokens[*self.stack.last().unwrap()].payload];
            if let Some(children) = element.children {
                self.children[children].push(index);
            } else {
                let tag = &self.tags[element.tag];
                abort!(tag.span(), "children are not allowed in self closing tags")
            }
        } else {
            self.root.push(index);
        }
    }

    fn parse_element(&mut self, input: ParseStream) -> syn::Result<()> {
        eat!(input, <);

        let mut name: Spanned<String> = Spanned::default();
        let mut decl: bool = false;

        let lookahead = input.lookahead1();
        on_peek!(lookahead {
            Token![/] => {
                eat!(input, /);
                name = self.parse_name(input)?;
                if self.stack.len() == 0 {
                    abort!(input.span(), "attempt to close a tag that was not opened"; help="try removing `</{}>`", name.value())
                }
                let last = self.stack.last().unwrap();
                let tag = self.get_tag(*last).unwrap();
                if name.value() != tag.value() {
                    abort!(name.span(), "unbalanced closing tags"; help="try closing `</{}>` first", tag.value())
                }
                self.stack.pop();
                eat!(input, >);
                return Ok(());
            };
            Token![!] => {
                eat!(input, !);
                if input.peek(Token![-]) {
                    eat!(input, --);
                    self.add_child(self.tokens.len());
                    self.tokens.push(token!(Comment, self.content.len()));
                    self.content.push(spanned!(input, @text));
                    eat!(input, -->);
                    return Ok(());
                } else {
                    decl = true;
                    name = self.parse_name(input)?;
                }
            };
            Ident::peek_any => {
                name = self.parse_name(input)?;
            };
            else => {
                let err = lookahead.error();
                abort!(err.span(), err.to_string());
            }
        });

        let tag = if self.tag_map.contains_key(name.value()) {
            self.tag_map.get(name.value()).unwrap().clone()
        } else {
            self.tag_map.insert(name.inner(), self.tags.len());
            self.tags.push(name);
            self.tags.len() - 1
        };

        let (attrs, spread, captures) = self.parse_attributes(input)?;

        self.add_child(self.tokens.len());
        self.tokens.push(token!(Element, self.elements.len()));

        if input.peek(Token![/]) {
            eat!(input, />);
            self.elements.push(Element {
                tag,
                attrs,
                spread,
                captures,
                children: None,
                decl,
            });
        } else {
            eat!(input, >);
            let children = if decl {
                None
            } else {
                self.stack.push(self.tokens.len() - 1);
                self.children.push(Vec::new());
                Some(self.children.len() - 1)
            };

            self.elements.push(Element {
                tag,
                attrs,
                spread,
                captures,
                children,
                decl,
            });
        }
        Ok(())
    }

    pub fn parse(&mut self, input: ParseStream) -> syn::Result<()> {
        while !input.is_empty() {
            on_peek!(input {
                    Token![<] => {
                        self.parse_element(input)?;
                    };
                    LitStr => {
                        self.add_child(self.tokens.len());
                        self.tokens.push(token!(Text, self.content.len()));
                        self.content.push(spanned!(input, @text));
                    };
                    Brace => {
                        self.add_child(self.tokens.len());
                        self.tokens.push(token!(Capture, self.captures.len()));
                        self.captures.push(spanned!(input, @capture));
                    };
                    else => {
                        abort!(input.span(), "Invalid syntax"; help="Try defining a string literal, comment, or an open tag")
                    }
                }
            );
            // abort!(input.span(), "Not yet finished");
        }

        if self.stack.len() > 0 {
            for index in self.stack[..self.stack.len() - 1].iter() {
                let tag = self.get_tag(*index).unwrap();
                emit_error!(tag.span(), "tag was never closed"; help="try adding `</{}>`", tag.value(); help="try making the tag self closing");
            }
            let last = self.get_tag(*self.stack.last().unwrap()).unwrap();
            abort!(last.span(), "tag was never closed"; help="try adding `</{}>`", last.value(); help="try making the tag self closing");
        }

        Ok(())
    }

    fn tokenize_text(&self, payload: &usize) -> (AET, TokenStream2) {
        let content = self.content[*payload].value();
        (AET::Push, quote!(Element::text(#content)))
    }

    fn tokenize_comment(&self, payload: &usize) -> (AET, TokenStream2) {
        let content = self.content[*payload].value();
        (AET::Push, quote!(Element::comment(#content)))
    }

    fn tokenize_capture(&self, payload: &usize) -> (AET, TokenStream2) {
        let capture = self.captures[*payload].value();
        (AET::OptionalExtend, quote!(#capture.into_children()))
    }

    fn tokenize_attrs(
        &self,
        index: &Option<usize>,
        captures: &Vec<usize>,
        spread: &Option<usize>,
    ) -> Option<TokenStream2> {
        let mut extends = Vec::new();
        if let Some(index) = index {
            let entries = &self.attrs[*index];

            let mut attrs = TokenStream2::new();
            for (name, value) in entries {
                let value = match value.value() {
                    Attribute::Exists => quote!("yes".to_string()),
                    Attribute::Literal(index) => {
                        let lit = &self.content[*index];
                        quote!(#lit.to_string())
                    }
                    Attribute::Capture(index) => {
                        let capture = self.captures[*index].inner();
                        quote!(#capture.to_prop())
                    }
                };
                attrs.append_all(quote!((#name.to_string(), #value),))
            }

            extends.push(quote!([#attrs]));
        }

        let mut caps = TokenStream2::new();
        for capture in captures {
            let capture = &self.captures[*capture];
            caps.append_all(quote!((#capture.to_string(), "yes".to_string()),))
        }

        if !caps.is_empty() {
            extends.push(quote!([#caps]));
        }

        if let Some(index) = spread {
            let spread = &self.spreads[*index];
            extends.push(quote!(#spread));
        }

        if extends.is_empty() {
            return None;
        }

        let first = extends.first().unwrap();
        if extends.len() == 1 {
            Some(quote!(std::collections::HashMap::from(#first)))
        } else {
            let result = extends[1..]
                .iter()
                .map(|e| quote!(_a.extend(#e);))
                .collect::<TokenStream2>();
            Some(quote!({
                let mut _a = std::collections::HashMap::from(#first);
                #result
                _a
            }))
        }
    }

    fn tokenize_for_element(
        &self,
        tag: &Spanned<String>,
        element: &Element,
    ) -> (AET, TokenStream2) {
        let mut lbinding: Option<String> = None;
        if let Some(index) = element.attrs {
            for (attr, value) in self.attrs[index].iter() {
                if attr.as_str().starts_with("let:") {
                    if None != lbinding || attr.len() < 4 {
                        abort!(
                        value.span(),
                        "Invalid let binding";
                        help="{}", if None != lbinding {
                            "Try removing the extra let binding"
                        } else {
                            "Try adding a name after `let:`"
                        }
                        );
                    }
                    lbinding = Some((&attr[4..]).to_string());
                }
            }
        }

        let mut found = false;
        let mut handler: Option<TokenStream2> = None;

        let mut before = TokenStream2::new();
        let mut after = TokenStream2::new();

        match element.children {
            None => {}
            Some(index) => {
                for child in self.children[index].iter() {
                    let token = &self.tokens[*child];
                    match token.ttype {
                        TokenType::Capture if !found => {
                            handler = Some(self.captures[token.payload].inner());
                            found = true;
                        }
                        _ => {
                            match self.tokenize(token) {
                                Some((e, t)) => {
                                    let chld = if let AET::OptionalExtend = e {
                                        quote!(match #t {
                                                Some(values) => _t.extend(values),
                                                None => {}
                                            };)
                                    } else if let AET::Extend = e {
                                        quote!(_t.extend(#t);)
                                    } else {
                                        quote!(_t.push(#t);)
                                    };

                                    if found {
                                        after.append_all(chld);
                                    } else {
                                        before.append_all(chld)
                                    }
                                }
                                None => {}
                            };
                        }
                    }
                }
            }
        };

        match handler {
            Some(handler) => {
                let binding = Ident::new(lbinding.unwrap().as_str(), Span::call_site());
                (
                    AET::Extend,
                    quote!(
                        {
                            let mut _t: Vec<Element> = Vec::new();
                            let _items = #binding.iter().map(#handler).collect::<Vec<Element>>();
                            for _item in _items {
                                #before
                                _t.push(_item);
                                #after
                            }
                            _t
                        }
                    ),
                )
            }
            None => {
                abort!(
                    tag.span(),
                    "Must have one child closure that is captured inside a `for` element"
                )
            }
        }
    }

    fn tokenize_element(&self, payload: &usize) -> (AET, TokenStream2) {
        let element = &self.elements[*payload];
        let decl = element.decl;
        let tag = &self.tags[element.tag];

        if tag.value().as_str() == "for" {
            self.tokenize_for_element(tag, element)
        } else if TAGS.contains(tag.value().as_str()) {
            // Match on return if None do something else take the Some(TokenStream)
            let attrs = self
                .tokenize_attrs(&element.attrs, &element.captures, &element.spread)
                .unwrap_or(quote!(None));

            let chldrn = match element.children {
                Some(children) => self.tokenize_children(
                    self.children[children]
                        .iter()
                        .filter_map(|c| self.tokenize(&self.tokens[*c]))
                        .collect(),
                ),
                None => {
                    quote!(None)
                }
            };

            (
                AET::Push,
                quote!(Element::tag(#decl, #tag, #attrs, #chldrn)),
            )
        } else {
            let attrs = self
                .tokenize_attrs(&element.attrs, &element.captures, &element.spread)
                .unwrap_or(quote!(std::collections::HashMap::new()));

            let chldrn = match element.children {
                Some(children) => self.tokenize_children(
                    self.children[children]
                        .iter()
                        .filter_map(|c| self.tokenize(&self.tokens[*c]))
                        .collect(),
                ),
                None => {
                    quote!(Vec::new())
                }
            };
            let tag = Ident::new(tag.value().as_str(), tag.span());
            (AET::Push, quote!(#tag.create_component(#attrs, #chldrn)))
        }
    }

    fn tokenize_children(&self, children: Vec<(AET, TokenStream2)>) -> TokenStream2 {
        match children.len() {
            0 => {
                quote!(Vec::new())
            }
            1 => {
                let child = children.first().unwrap();
                let content = child.1.clone();
                match child.0 {
                    AET::Push => quote!(Vec::from([#content])),
                    AET::Extend => quote!(#content),
                    AET::OptionalExtend => quote!(match #content {
                        Some(_value) => _value,
                        None => Vec::new()
                    }),
                }
            }
            _ => {
                let mut chldrn = TokenStream2::new();
                for (target, content) in children.iter() {
                    match target {
                        AET::Push => chldrn.append_all(quote!(_t.push(#content);)),
                        AET::Extend => chldrn.append_all(quote!(_t.extend(#content);)),
                        AET::OptionalExtend => chldrn.append_all(quote!(match #content {
                            Some(_value) => _t.extend(_value),
                            None => {}
                        };)),
                    }
                }
                quote!({
                    let mut _t = Vec::new();
                    #chldrn
                    _t
                })
            }
        }
    }

    fn tokenize(&self, token: &Token) -> Option<(AET, TokenStream2)> {
        match token.ttype {
            TokenType::None => None,
            TokenType::Text => Some(self.tokenize_text(&token.payload)),
            TokenType::Comment => Some(self.tokenize_comment(&token.payload)),
            TokenType::Capture => Some(self.tokenize_capture(&token.payload)),
            TokenType::Element => Some(self.tokenize_element(&token.payload)),
        }
    }
}
