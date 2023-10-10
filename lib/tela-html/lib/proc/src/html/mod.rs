use proc_macro2::Span;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
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
                Self::Yes => "true".to_string(),
                Self::Capture(val) => format!("{{{}}}", val),
                Self::Value(val) => format!("{:?}", val),
            }
        )
    }
}

/// Represenative of an html element
#[derive(Clone)]
pub enum Element {
    Capture(TokenStream2),
    Text(String),
    Comment(String),
    Tag {
        tag: String,
        attrs: HashMap<String, Attribute>,
        spread: Option<String>,
        captures: Vec<TokenStream2>,
        children: Option<Vec<Rc<RefCell<Element>>>>,
        parent: Rc<RefCell<Element>>,
    },
    Root(Vec<Rc<RefCell<Element>>>),
}

lazy_static::lazy_static! {
    static ref TAGS: HashSet<&'static str> = HashSet::from([
        "a",
        "abbr",
        "address",
        "area",
        "article",
        "aside",
        "audio",
        "b",
        "base",
        "bdi",
        "bdo",
        "blockquote",
        "body",
        "br",
        "button",
        "canvas",
        "caption",
        "cite",
        "code",
        "col",
        "colgroup",
        "data",
        "datalist",
        "dd",
        "del",
        "details",
        "dfn",
        "dialog",
        "div",
        "dl",
        "dt",
        "em",
        "embed",
        "fieldset",
        "figure",
        "footer",
        "form",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "head",
        "header",
        "hgroup",
        "hr",
        "html",
        "i",
        "iframe",
        "img",
        "input",
        "ins",
        "kbd",
        "keygen",
        "label",
        "legend",
        "li",
        "link",
        "main",
        "map",
        "mark",
        "menu",
        "menuitem",
        "meta",
        "meter",
        "nav",
        "noscript",
        "object",
        "ol",
        "optgroup",
        "option",
        "output",
        "p",
        "param",
        "pre",
        "progress",
        "q",
        "rb",
        "rp",
        "rt",
        "rtc",
        "ruby",
        "s",
        "samp",
        "script",
        "section",
        "select",
        "small",
        "source",
        "span",
        "strong",
        "style",
        "sub",
        "summary",
        "sup",
        "table",
        "tbody",
        "td",
        "template",
        "textarea",
        "tfoot",
        "th",
        "thead",
        "time",
        "title",
        "tr",
        "track",
        "u",
        "ul",
        "var",
        "video",
        "wbr",
    ]);
}

impl Element {
    pub fn attributes(&self) -> String {
        let mut result = Vec::new();
        if let Element::Tag {
            attrs,
            spread,
            captures,
            ..
        } = self
        {
            for (name, value) in attrs.iter() {
                result.push(format!(
                    "{}{}",
                    name,
                    match value {
                        Attribute::Value(val) => format!("={:?}", val),
                        Attribute::Capture(_) => "={}".to_string(),
                        _ => String::new(),
                    }
                ))
            }

            for _ in captures.iter() {
                result.push("{}".to_string())
            }

            match spread {
                Some(_) => result.push("{}".to_string()),
                None => {}
            };
        }
        if result.len() > 0 {
            result.insert(0, String::new());
        }
        result.join(" ")
    }

    pub fn is_component(&self) -> bool {
        if let Element::Tag { tag, .. } = self {
            return TAGS.contains(tag.as_str());
        }
        false
    }

    pub fn args(&self) -> Option<String> {
        match self {
            Element::Capture(val) => Some(format!("{{{}}}", val)),
            Element::Tag {
                tag,
                attrs,
                spread,
                captures,
                children,
                ..
            } => {
                if self.is_component() {
                    // Anything that is capture, spread, children that are not html tags
                    let mut result = Vec::new();
                    // attrs with capture
                    for (_, value) in attrs.iter() {
                        if let Attribute::Capture(cap) = value {
                            result.push(format!("{{{}}}", cap))
                        }
                    }
                    // Capture free attrs
                    for capture in captures {
                        result.push(format!("{{{}}}", capture))
                    }
                    // spread
                    match spread {
                        Some(spread) => result.push(format!("{}.to_attributes()", spread)),
                        None => {}
                    };
                    // children
                    if let Some(children) = children {
                        for child in children {
                            if let Some(args) = child.borrow().args() {
                                result.push(args)
                            }
                        }
                    }

                    if result.len() > 0 {
                        Some(result.join(", "))
                    } else {
                        None
                    }
                } else {
                    // Props
                    Some(format!(
                        "{}.create_component(Props::from(([{}], [{}], [{}], {})))",
                        tag.replace("-", "_"),
                        attrs
                            .iter()
                            .map(|(name, value)| format!(
                                "({:?}, Box::new({:?}))",
                                name.to_string(),
                                value.to_string()
                            ))
                            .collect::<Vec<String>>()
                            .join(", "),
                        captures
                            .iter()
                            .map(|v| format!("Box::new({{{}}})", v))
                            .collect::<Vec<String>>()
                            .join(", "),
                        match children {
                            Some(children) => {
                                children
                                    .iter()
                                    .filter_map(|v| {
                                        if v.borrow().is_component() {
                                            v.borrow().args()
                                        } else {
                                            v.borrow().args().map(|r| {
                                                format!(
                                                    r#"format!({:?}, {})"#,
                                                    v.borrow().to_string(),
                                                    r
                                                )
                                            })
                                        }
                                    })
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            }
                            None => String::new(),
                        },
                        match spread {
                            Some(spread) => format!("Some({})", spread),
                            None => "None::<Vec<(&str, &str)>>".to_string(),
                        }
                    ))
                }
            }
            _ => None,
        }
    }

    pub fn display(&self, offset: usize) -> String {
        let indent = (0..offset).map(|_| ' ').collect::<String>();
        match self {
            Element::Capture(_) => indent + "{}",
            Element::Comment(_) => String::new(),
            Element::Text(text) => indent + text,
            // Root shouldn't be nested so it will be ignored
            Element::Root(_) => String::new(),
            Element::Tag { tag, children, .. } => {
                if self.is_component() {
                    let (sc, c, ct) = match children {
                        None => (" /", String::new(), String::new()),
                        Some(children) if children.len() == 0 => {
                            ("", String::new(), format!("</{}>", tag))
                        }
                        Some(children) => (
                            "",
                            String::from("\n")
                                + children
                                    .iter()
                                    .map(|c| c.borrow().display(offset + 4))
                                    .collect::<Vec<String>>()
                                    .join("\n")
                                    .as_str(),
                            format!("\n{}</{}>", indent, tag),
                        ),
                    };

                    format!(
                        r#"{indent}<{}{}{}>{}{}"#,
                        tag,
                        self.attributes(),
                        sc,
                        c,
                        ct,
                        indent = indent
                    )
                } else {
                    format!("{}{{}}", indent)
                }
            }
        }
    }
    pub fn debug(&self, offset: usize) -> String {
        let indent = (0..offset).map(|_| ' ').collect::<String>();
        match self {
            Element::Capture(_) => indent + "Capture",
            Element::Comment(_) => indent + "Comment",
            Element::Text(_) => indent + "Text",
            Element::Root(_) => indent + "Root",
            Element::Tag {
                tag: name,
                attrs,
                spread,
                captures,
                children,
                ..
            } => {
                format!(
                    r#"{indent}Element::{}({}){}{}{}{}"#,
                    name,
                    match children {
                        Some(cldrn) => cldrn.len(),
                        None => 0,
                    },
                    if attrs.len() > 0 {
                        format!("\n{}  - attrs: {}", indent, attrs.len())
                    } else {
                        String::new()
                    },
                    if captures.len() > 0 {
                        format!("\n{}  - captures: {}", indent, captures.len())
                    } else {
                        String::new()
                    },
                    match spread {
                        Some(spread) => format!("\n{}  - Spread: {:?}", indent, spread),
                        None => String::new(),
                    },
                    match children {
                        Some(children) => {
                            String::from("\n")
                                + children
                                    .iter()
                                    .map(|c| c.borrow().debug(offset + 2))
                                    .collect::<Vec<String>>()
                                    .join("\n")
                                    .as_str()
                        }
                        None => String::new(),
                    },
                    indent = indent,
                )
            }
        }
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display(0))
    }
}

impl Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.debug(0))
    }
}

#[derive(Clone)]
pub struct Segment {
    children: Vec<Rc<RefCell<Element>>>,
}

impl Debug for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for element in self.children.iter() {
            write!(f, "{:?}\n", element.borrow())?;
        }
        Ok(())
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for element in self.children.iter() {
            write!(f, "{}\n", element.borrow())?;
        }
        Ok(())
    }
}

impl ToTokens for Segment {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut result = Vec::new();
        for child in self.children.iter() {
            result.push(format!("{}", child.borrow()));
        }
        let result = result.join("\n");
        let args = self.args().parse::<TokenStream2>().unwrap();
        tokens.append_all(quote! {
            {
                use tela_html::{ToAttributes, ToAttrValue, Props, Component};

                format!(
                    #result,
                    #args
                )
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
    fn args(&self) -> String {
        self.children
            .iter()
            .filter_map(|c| {
                let args = c.borrow().args();
                args
            })
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn parse_comment(input: ParseStream) -> syn::Result<Element> {
        Eat![input, !--];
        let result = Element::Comment(input.parse::<LitStr>()?.value());
        Eat![input, -->];
        Ok(result)
    }

    fn close_element(input: ParseStream, stack: &mut Vec<String>) -> syn::Result<()> {
        input.parse::<Token![/]>()?;
        let mut name = Vec::new();
        let first = input.parse::<Ident>()?;
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
            name.push(input.parse::<Ident>()?.to_string())
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

    fn parse_attr(input: ParseStream) -> syn::Result<(String, Attribute)> {
        let mut name = input.parse::<Ident>()?.to_string();
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
            name.push_str(input.parse::<Ident>()?.to_string().as_str())
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
        Ok((name, value))
    }

    fn parse_props_attrs(
        input: ParseStream,
    ) -> syn::Result<(
        HashMap<String, Attribute>,
        Vec<TokenStream2>,
        Option<String>,
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
                    spread = Some(braces.parse::<Ident>()?.to_string());
                } else {
                    captures.push(braces.parse::<TokenStream2>()?)
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
        let mut name = vec![input.parse::<Ident>()?.to_string()];
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
            name.push(input.parse::<Ident>()?.to_string())
        }

        let name = name.join("-");
        let (attrs, captures, spread) = Segment::parse_props_attrs(input)?;

        if input.peek(Token![/]) {
            Eat![input, />];
            Segment::append(
                parent.clone(),
                Element::Tag {
                    tag: name,
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
                    tag: name,
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
