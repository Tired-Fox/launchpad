use proc_macro2::Span;
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    rc::Rc,
};

use proc_macro2::TokenStream as TokenStream2;

use proc_macro_error::{abort, emit_call_site_error};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    braced,
    ext::IdentExt,
    parse::{ParseBuffer, ParseStream},
    token::Brace,
    Ident, LitStr, Token,
};

mod constants;
use constants::TAGS;

/// An element attribute value
#[derive(Debug, Clone)]
pub enum Attribute {
    Yes,
    Capture(TokenStream2),
    Value(String),
}

impl Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Yes => String::from("\"yes\""),
                Self::Capture(capture) => capture.to_string(),
                Self::Value(value) => format!("{:?}", value),
            }
        )
    }
}

#[derive(Clone)]
pub struct Spanned<T: Clone>(pub Span, pub T);
impl<T: Clone> Spanned<T> {
    pub fn span(&self) -> &Span {
        &self.0
    }

    pub fn inner(&self) -> &T {
        &self.1
    }
}

impl<T: Clone + Display> Display for Spanned<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner())
    }
}

/// Represenative of an html element
#[derive(Clone)]
pub enum Element {
    Capture(TokenStream2),
    Text(String),
    Comment(String),
    Tag {
        decl: bool,
        tag: Spanned<String>,
        attrs: HashMap<String, Spanned<Attribute>>,
        spread: Option<Spanned<TokenStream2>>,
        captures: Vec<Spanned<TokenStream2>>,
        children: Option<Vec<Rc<RefCell<Element>>>>,
        parent: Rc<RefCell<Element>>,
    },
    Root(Vec<Rc<RefCell<Element>>>),
}

/// Append Element Type
enum AET {
    Push,
    Extend,
    OptionalExtend,
}

impl Element {
    fn to_token_stream(&self) -> (AET, TokenStream2) {
        let mut tokens = TokenStream2::new();
        let mut extend = AET::Push;
        match self {
            Self::Tag {
                tag,
                attrs,
                spread,
                captures,
                children,
                decl,
                ..
            } => {
                let mut attributes = String::from("[");
                for (name, value) in attrs.iter() {
                    attributes.push_str(
                        format!("({:?}.to_string(), {}.to_prop()),", name, value.to_string())
                            .as_str(),
                    );
                }
                for capture in captures {
                    attributes.push_str(
                        format!(r#"({}.to_string(), "yes".to_string()),"#, capture.inner())
                            .as_str(),
                    )
                }
                attributes.push_str("]");

                if let Some(spread) = spread {
                    if attributes.len() > 2 {
                        attributes = format!(
                            r#"{{let mut attrs = {}.into_attrs();attrs.extend({});attrs}}"#,
                            spread, attributes
                        );
                    } else {
                        attributes = spread.to_string()
                    }
                }

                if tag.inner().as_str() == "for" {
                    let mut lbinding: Option<String> = None;
                    for (attr, value) in attrs.iter() {
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

                    let mut found = false;
                    let mut handler: Option<TokenStream2> = None;

                    let mut before = TokenStream2::new();
                    let mut after = TokenStream2::new();

                    match children {
                        None => {}
                        Some(children) => {
                            for child in children {
                                match &*child.borrow() {
                                    Element::Capture(capture) if !found => {
                                        handler = Some(capture.clone());
                                        found = true;
                                    }
                                    other => {
                                        let (e, t) = other.to_token_stream();
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
                                }
                            }
                        }
                    };

                    match handler {
                        Some(handler) => {
                            let binding = Ident::new(lbinding.unwrap().as_str(), Span::call_site());
                            tokens.append_all(quote!(
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
                            ));
                            extend = AET::Extend;
                        }
                        None => {
                            abort!(
                                tag.span(),
                                "Must have one child closure that is captured inside a `for` element"
                            )
                        }
                    }
                } else if TAGS.contains(tag.inner().as_str()) {
                    if attributes.len() == 2 {
                        attributes = "None".to_string();
                    }

                    let attributes = attributes.parse::<TokenStream2>().unwrap_or(quote!(None));

                    let chldrn = match children {
                        None => quote!(None),
                        Some(children) => {
                            let mut chts = TokenStream2::new();
                            for child in children {
                                let (e, t) = child.borrow().to_token_stream();
                                if let AET::OptionalExtend = e {
                                    chts.append_all(quote!(match #t {
                                        Some(values) => _t.extend(values),
                                        None => {}
                                    };))
                                } else if let AET::Extend = e {
                                    chts.append_all(quote!(_t.extend(#t);))
                                } else {
                                    chts.append_all(quote!(_t.push(#t);))
                                }
                            }
                            quote!({
                                let mut _t: Vec<Element> = Vec::new();
                                #chts
                                _t
                            })
                        }
                    };

                    let tag = tag.inner();
                    tokens.append_all(quote! {
                        Element::tag(#decl, #tag, #attributes, #chldrn)
                    })
                } else {
                    let tag = tag
                        .inner()
                        .replace("-", "_")
                        .parse::<TokenStream2>()
                        .unwrap();

                    let chldrn = match children {
                        None => quote!(Vec::new()),
                        Some(children) => {
                            let mut chts = TokenStream2::new();
                            for child in children {
                                let (e, t) = child.borrow().to_token_stream();
                                if let AET::OptionalExtend = e {
                                    chts.append_all(quote!(match #t {
                                        Some(values) => _t.extend(values),
                                        None => {}
                                    };))
                                } else if let AET::Extend = e {
                                    chts.append_all(quote!(_t.extend(#t);))
                                } else {
                                    chts.append_all(quote!(_t.push(#t);))
                                }
                            }
                            quote!({
                                let mut _t: Vec<Element> = Vec::new();
                                #chts
                                _t
                            })
                        }
                    };

                    let attributes = attributes.parse::<TokenStream2>().unwrap();
                    tokens.append_all(quote!(
                        #tag.create_component(#attributes.into_attrs(), #chldrn)
                    ));
                }
            }
            Self::Text(text) => tokens.append_all(quote!(Element::text(#text))),
            Self::Comment(comment) => tokens.append_all(quote!(Element::comment(#comment))),
            Self::Capture(capture) => {
                tokens.append_all(quote!({#capture}.into_children()));
                extend = AET::OptionalExtend;
            }
            _ => {}
        };
        (extend, tokens)
    }
}

impl Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Tag { tag, .. } => tag.to_string(),
                Self::Text(_) => String::from("Text"),
                Self::Comment(_) => String::from("Comment"),
                Self::Capture(_) => String::from("Capture"),
                Self::Root(_) => String::from("Root"),
            }
        )
    }
}

#[derive(Clone)]
pub struct Segment {
    children: Vec<Rc<RefCell<Element>>>,
}

impl ToTokens for Segment {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let children: Vec<(AET, TokenStream2)> = self
            .children
            .iter()
            .map(|c| c.borrow().to_token_stream())
            .collect();

        let default = quote!(Element::None);
        let result = if children.len() == 0 {
            default
        } else if children.len() > 1 {
            let mut chts = TokenStream2::new();
            for child in children {
                let result = child.1;
                if let AET::OptionalExtend = child.0 {
                    chts.append_all(quote!(match #result {
                        Some(values) => _t.extend(values),
                        None => {}
                    };));
                } else if let AET::Extend = child.0 {
                    chts.append_all(quote!(_t.extend(#result);))
                } else {
                    chts.append_all(quote!(_t.push(#result);))
                }
            }
            quote!(
                Element::Wrapper({
                    let mut _t: Vec<Element> = Vec::new();
                    #chts
                    _t
                })
            )
        } else {
            let first = children.first().map(|v| v.1.clone()).unwrap_or(default);
            quote!(#first)
        };

        tokens.append_all(quote! {
            {
                use tela_html::prelude::*;
                #result
            }
        });
    }
}

macro_rules! Eat {
    [$name: ident, $($symbol: tt)*] => {
        {
            $(let _ = $name.parse::<Token![$symbol]>();)*
        }
    };
}

impl Segment {
    fn parse_comment(input: ParseStream) -> syn::Result<Element> {
        Eat![input, !--];
        let result = Element::Comment(input.parse::<LitStr>()?.value());
        Eat![input, -->];
        Ok(result)
    }

    fn close_element(input: ParseStream, stack: &mut Vec<String>) -> syn::Result<()> {
        input.parse::<Token![/]>()?;
        let mut name = Vec::new();
        let first = Ident::parse_any(input)?;
        name.push(first.to_string());
        loop {
            if input.peek(Token![-]) {
                input.parse::<Token![-]>()?;
                name.push("-".to_string())
            } else if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
                name.push(":".to_string())
            } else {
                break;
            }
            name.push(Ident::parse_any(input)?.to_string())
        }
        Eat![input, >];

        // Do logic checks on closing tag
        if stack.len() == 0 {
            abort!(first, "Cannot close {} because it was never opened", name.join("-"); help="Try adding <{}>", name.join("-"))
        }

        let last = stack.last().unwrap();
        if last != &name.join("-") {
            let mut to_close = Vec::new();
            for tag in stack.iter().rev().skip(1) {
                if tag == last {
                    abort!(first, "Unexpected closing tag"; help="Make sure to close the following tags first: [{}]", to_close.join(", "))
                }
                to_close.push(tag.clone());
            }
            abort!(first, "Unexpected closing tag"; help="Make sure the tag was first opened")
        } else {
            stack.pop();
        }
        Ok(())
    }

    fn parse_attr(input: ParseStream) -> syn::Result<(String, Spanned<Attribute>)> {
        let first = Ident::parse_any(input)?;
        let mut name = first.to_string();
        loop {
            if input.peek(Token![-]) {
                input.parse::<Token![-]>()?;
                name.push_str("-")
            } else if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
                name.push_str(":")
            } else {
                break;
            }
            name.push_str(Ident::parse_any(input)?.to_string().as_str())
        }

        let value = if input.peek(Token![=]) {
            let equal = input.parse::<Token![=]>()?;
            // String literal
            if input.peek(LitStr) {
                Attribute::Value(input.parse::<LitStr>()?.value())
                // Expr block (Capture)
            } else if input.peek(Brace) {
                let braces: ParseBuffer;
                braced!(braces in input);
                Attribute::Capture(braces.parse::<TokenStream2>()?)
            } else {
                abort!(equal, "Expected string literal or expression block")
            }
        } else {
            Attribute::Yes
        };
        Ok((name, Spanned(first.span(), value)))
    }

    fn parse_props_attrs(
        input: ParseStream,
    ) -> syn::Result<(
        HashMap<String, Spanned<Attribute>>,
        Vec<Spanned<TokenStream2>>,
        Option<Spanned<TokenStream2>>,
    )> {
        // Start with:
        // - Block ({...spread}) ~ Must contain an ellipse and then an ident
        // - Colon (:) ~ Must be followed by an ident
        // - Ident

        // Value:
        // - Literal ("")
        // - Standalone
        // - Block ({})
        let mut attrs = HashMap::new();
        let mut captures = Vec::new();
        let mut spread = None;
        while !input.peek(Token![/]) && !input.peek(Token![>]) && !input.is_empty() {
            if input.peek(Ident::peek_any) {
                let (name, value) = Segment::parse_attr(input)?;
                attrs.insert(name, value);
            } else if input.peek(Brace) {
                let braces;
                braced!(braces in input);
                if braces.peek(Token![..]) {
                    braces.parse::<Token![..]>()?;
                    spread = Some(Spanned(braces.span(), braces.parse::<TokenStream2>()?));
                } else {
                    captures.push(Spanned(braces.span(), braces.parse::<TokenStream2>()?));
                }
            } else {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "invalid attribute syntax",
                ));
            }
        }
        Ok((attrs, captures, spread))
    }

    fn parse_element(
        input: ParseStream,
        stack: &mut Vec<String>,
        parent: Rc<RefCell<Element>>,
    ) -> syn::Result<Rc<RefCell<Element>>> {
        let decl = input.peek(Token![!]);
        let _ = input.parse::<Token![!]>();
        let first = Ident::parse_any(input)?;
        let mut name = first.to_string();
        loop {
            if input.peek(Token![-]) {
                input.parse::<Token![-]>()?;
                name.push_str("-")
            } else if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
                name.push_str(":")
            } else {
                break;
            }
            name.push_str(Ident::parse_any(input)?.to_string().as_str())
        }

        let (attrs, captures, spread) = Segment::parse_props_attrs(input)?;

        if decl {
            Eat![input, >];
            Segment::append(
                parent.clone(),
                Element::Tag {
                    decl: true,
                    tag: Spanned(first.span(), name),
                    attrs,
                    captures,
                    spread,
                    children: None,
                    parent: parent.clone(),
                },
            );
            Ok(parent.clone())
        } else if input.peek(Token![/]) {
            Eat![input, />];
            Segment::append(
                parent.clone(),
                Element::Tag {
                    decl: false,
                    tag: Spanned(first.span(), name),
                    attrs,
                    captures,
                    spread,
                    children: None,
                    parent: parent.clone(),
                },
            );
            Ok(parent.clone())
        } else {
            Eat![input, >];
            stack.push(name.clone());
            Ok(Segment::append(
                parent.clone(),
                Element::Tag {
                    decl: false,
                    tag: Spanned(first.span(), name),
                    attrs,
                    captures,
                    spread,
                    children: Some(Vec::new()),
                    parent: parent.clone(),
                },
            ))
        }
    }

    fn previous(
        parent: Rc<RefCell<Element>>,
        default: Rc<RefCell<Element>>,
    ) -> Rc<RefCell<Element>> {
        match &mut *parent.borrow_mut() {
            Element::Tag { parent: pp, .. } => pp.clone(),
            _ => default,
        }
    }

    fn append(parent: Rc<RefCell<Element>>, child: Element) -> Rc<RefCell<Element>> {
        let next = Rc::new(RefCell::new(child));
        match &mut *parent.borrow_mut() {
            Element::Tag {
                children: Some(children),
                ..
            } => {
                children.push(next.clone());
            }
            Element::Root(children) => {
                children.push(next.clone());
            }
            _ => emit_call_site_error!("Invalid parent element {:?}", parent),
        };
        next
    }
}

impl syn::parse::Parse for Segment {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let mut stack: Vec<String> = Vec::new();
        let root = Rc::new(RefCell::new(Element::Root(Vec::new())));
        let mut parent: Rc<RefCell<Element>> = root.clone();

        while !input.is_empty() {
            if input.peek(LitStr) {
                Segment::append(
                    parent.clone(),
                    Element::Text(input.parse::<LitStr>()?.value()),
                );
            } else if input.peek(Brace) {
                let braces;
                braced!(braces in input);
                Segment::append(
                    parent.clone(),
                    Element::Capture(braces.parse::<TokenStream2>()?),
                );
            } else if input.peek(Token![<]) {
                let _ = input.parse::<Token![<]>()?;
                if input.peek(Token![!]) {
                    Segment::append(parent.clone(), Segment::parse_comment(input)?);
                } else if input.peek(Ident::peek_any) {
                    parent = Segment::parse_element(input, &mut stack, parent.clone())?;
                } else if input.peek(Token![/]) {
                    // Parse whole closing tag
                    Segment::close_element(input, &mut stack)?;
                    parent = Segment::previous(parent.clone(), root.clone());
                }
            } else {
                abort!(input.span(), "Invalid syntax"; help="Expected `<!--` comment or `<{name}` tag")
            }
        }

        if let Element::Root(children) = &*root.borrow() {
            return Ok(Segment {
                children: children.to_owned(),
            });
        }
        Ok(Segment {
            children: Vec::new(),
        })
    }
}
