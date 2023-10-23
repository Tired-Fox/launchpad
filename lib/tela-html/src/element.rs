use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use crate::{escape, ToAttrValue};

macro_rules! or_new {
    (($($first: tt)*) $(=> ($($condition: tt)*))*, $($result: tt)*) => {
       if $($first)* {
           or_new!{@block {$($result)*}, $(($($condition)*),)*}
       } else {String::new()}
    };
    (@block {$($result:tt)*}, ($($condition: tt)*), $($rest: tt)*) => {
       if $($condition)* {
           or_new!{@block {$($result)*}, $($rest)*}
       } else {String::new()}
    };
    (@block {$($result:tt)*},) => {
        $($result)*
    }
}

pub trait IntoAttrs {
    fn into_attrs(self) -> HashMap<String, String>;
}

impl<A: Display, B: Display, const SIZE: usize> IntoAttrs for [(A, B); SIZE] {
    fn into_attrs(self) -> HashMap<String, String> {
        self.iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect()
    }
}

impl<A: Display, B: Display> IntoAttrs for &[(A, B)] {
    fn into_attrs(self) -> HashMap<String, String> {
        self.iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect()
    }
}

impl<A: Display, B: Display> IntoAttrs for Vec<(A, B)> {
    fn into_attrs(self) -> HashMap<String, String> {
        self.iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect()
    }
}

impl<A: Display, B: Display> IntoAttrs for HashMap<A, B> {
    fn into_attrs(self) -> HashMap<String, String> {
        self.iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect()
    }
}

impl IntoAttrs for Option<&[(&str, &str)]> {
    fn into_attrs(self) -> HashMap<String, String> {
        match self {
            Some(v) => v.into_attrs(),
            None => HashMap::new(),
        }
    }
}

pub trait IntoChildren<T = ()> {
    fn into_children(&self) -> Option<Vec<Element>>;
}

impl<F> IntoChildren<Box<dyn FnOnce() -> Element>> for F
where
    F: FnOnce() -> Element + Clone,
{
    fn into_children(&self) -> Option<Vec<Element>> {
        Some(vec![(self.clone())()])
    }
}

impl<T: Display> IntoChildren<Box<dyn Display>> for T {
    fn into_children(&self) -> Option<Vec<Element>> {
        Some(Vec::from([Element::Text(escape(self.to_string()))]))
    }
}

impl IntoChildren for () {
    fn into_children(&self) -> Option<Vec<Element>> {
        None
    }
}

impl IntoChildren for Option<Vec<Element>> {
    fn into_children(&self) -> Option<Vec<Element>> {
        self.clone()
    }
}

impl IntoChildren for Element {
    fn into_children(&self) -> Option<Vec<Element>> {
        Some(Vec::from([self.clone()]))
    }
}

impl IntoChildren for Vec<Element> {
    fn into_children(&self) -> Option<Vec<Element>> {
        Some(self.clone())
    }
}

#[derive(Clone)]
pub enum Element {
    None,
    Wrapper(Vec<Element>),
    Comment(String),
    Text(String),
    Tag {
        decl: bool,
        tag: String,
        attrs: HashMap<String, String>,
        children: Option<Vec<Element>>,
    },
}

impl Element {
    pub fn text<S: Display>(text: S) -> Element {
        Element::Text(text.to_string())
    }

    pub fn comment<S: Display>(comment: S) -> Element {
        Element::Comment(comment.to_string())
    }

    pub fn wrapper<C: IntoIterator<Item = Element>>(children: C) -> Element {
        Element::Wrapper(children.into_iter().collect())
    }

    pub fn tag<S, A, C>(decl: bool, tag: S, attrs: A, children: C) -> Element
    where
        S: Display,
        A: IntoAttrs,
        C: IntoChildren,
    {
        Element::Tag {
            decl,
            tag: tag.to_string(),
            attrs: attrs.into_attrs(),
            children: children.into_children(),
        }
    }
}

fn debug(element: &Element, offset: usize) -> Option<String> {
    let indent = (0..offset).map(|_| ' ').collect::<String>();
    match element {
        Element::None => None,
        Element::Wrapper(children) => Some(
            String::from("\n")
                + children
                    .iter()
                    .filter_map(|v| debug(v, offset))
                    .collect::<Vec<String>>()
                    .join("\n")
                    .as_str(),
        ),
        Element::Text(val) => Some(format!("{indent}Text({})", val.len(), indent = indent)),
        #[allow(unused_variables)]
        Element::Comment(val) => {
            #[cfg(feature = "comments")]
            return Some(format!("{indent}Comment({})", val.len(), indent = indent));
            #[cfg(not(feature = "comments"))]
            return None;
        }
        Element::Tag {
            decl,
            tag,
            attrs,
            children,
        } => Some(format!(
            "{indent}Element::{}{}{}{}",
            if *decl { "!" } else { "" },
            tag,
            or_new!(
                (attrs.len() > 0),
                if attrs.len() <= 2 {
                    format!(
                        "\n{indent} {{ {} }}",
                        attrs
                            .iter()
                            .map(|(name, value)| format!("{}: {:?}", name, value))
                            .collect::<Vec<String>>()
                            .join(", "),
                        indent = indent
                    )
                } else {
                    format!(
                        "\n{indent} {{\n{}\n{indent} }}",
                        attrs
                            .iter()
                            .map(|(name, value)| format!("{}   {}: {:?},", indent, name, value))
                            .collect::<Vec<String>>()
                            .join("\n"),
                        indent = indent
                    )
                }
            ),
            or_new!((let Some(children)=children) => (children.len() > 0),
                String::from("\n") + children.iter()
                    .filter_map(|v| debug(v, offset + 2))
                    .collect::<Vec<String>>()
                    .join("\n").as_str()
            ),
            indent = indent
        )),
    }
}

fn display(element: &Element) -> Option<String> {
    match element {
        Element::None => None,
        Element::Wrapper(children) => Some(format!("{}", {
            children
                .iter()
                .filter_map(|c| display(c))
                .collect::<Vec<String>>()
                .join("")
        })),
        Element::Text(val) => Some(val.clone()),
        #[allow(unused_variables)]
        Element::Comment(val) => {
            #[cfg(feature = "comments")]
            return Some(format!("<!-- {} -->", val));
            #[cfg(not(feature = "comments"))]
            return None;
        }
        Element::Tag {
            decl,
            tag,
            attrs,
            children,
        } => Some(format!(
            "<{}{}{}{}>{}{}",
            if *decl { "!" } else { "" },
            tag,
            or_new!(
                (attrs.len() > 0),
                format!(
                    " {}",
                    attrs
                        .iter()
                        .filter_map(|(name, value)| value
                            .to_attr_value()
                            .map(|val| format!("{}{}", name, val)))
                        .collect::<Vec<String>>()
                        .join(" "),
                )
            ),
            if let None = children {
                if !*decl {
                    " /"
                } else {
                    ""
                }
            } else {
                ""
            },
            or_new!((let Some(children)=children) => (children.len() > 0),
                children.iter().filter_map(|c| display(c)).collect::<Vec<String>>().join("")
            ),
            or_new!((let Some(_) = children),
                format!("</{}>", tag)
            ),
        )),
    }
}

impl From<Element> for Element {
    fn from(value: Element) -> Self {
        value
    }
}

impl Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(result) = debug(self, 0) {
            write!(f, "{}", result)?
        }
        Ok(())
    }
}

impl ToString for Element {
    fn to_string(&self) -> String {
        display(self).unwrap_or(String::new())
    }
}
