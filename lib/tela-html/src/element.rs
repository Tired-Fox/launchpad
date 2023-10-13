use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use crate::ToAttrValue;

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

pub trait IntoChildren {
    fn into_children(&self) -> Option<Vec<Element>>;
}

macro_rules! into_children {
    ([$($name: ty),* $(,)?]) => {
        $(
            impl IntoChildren for $name {
                fn into_children(&self) -> Option<Vec<Element>> {
                    Some(vec![Element::text(self)])
                }
            }
        )*
    };
}

into_children!([i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, String, &str]);

impl<F> IntoChildren for F
where
    F: FnOnce() -> Element + Clone,
{
    fn into_children(&self) -> Option<Vec<Element>> {
        Some(vec![(self.clone())()])
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

enum Type {
    Text,
    Other,
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

    fn debug(&self, offset: usize) -> Option<String> {
        let indent = (0..offset).map(|_| ' ').collect::<String>();
        match self {
            Self::None => None,
            Self::Wrapper(children) => Some(
                String::from("\n")
                    + children
                        .iter()
                        .filter_map(|v| v.debug(offset))
                        .collect::<Vec<String>>()
                        .join("\n")
                        .as_str(),
            ),
            Self::Text(val) => Some(format!("{indent}Text({})", val.len(), indent = indent)),
            #[allow(unused_variables)]
            Self::Comment(val) => {
                #[cfg(feature = "comments")]
                return Some(format!("{indent}Comment({})", val.len(), indent = indent));
                #[cfg(not(feature = "comments"))]
                return None;
            }
            Self::Tag {
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
                        .filter_map(|v| v.debug(offset + 2))
                        .collect::<Vec<String>>()
                        .join("\n").as_str()
                ),
                indent = indent
            )),
        }
    }

    fn etype(&self) -> Type {
        match self {
            Self::Text(_) => Type::Text,
            _ => Type::Other,
        }
    }

    fn display(&self, offset: usize) -> Option<String> {
        let indent = (0..offset).map(|_| ' ').collect::<String>();
        match self {
            Self::None => None,
            Self::Wrapper(children) => Some(format!(
                "{indent}{}",
                {
                    let mut result = Vec::new();
                    let mut previous = Type::Other;
                    for child in children.iter() {
                        let (lead, value) = if let Type::Other = child.etype() {
                            ("\n", child.display(offset))
                        } else if let Type::Other = previous {
                            ("\n", child.display(offset))
                        } else {
                            ("", child.display(0))
                        };

                        if let Some(value) = value {
                            previous = child.etype();
                            result.push(format!("{}{}", lead, value));
                        }
                    }
                    result.join("").trim_start().to_string()
                },
                indent = indent,
            )),
            Self::Text(val) => Some(format!("{}{}", indent, val)),
            #[allow(unused_variables)]
            Self::Comment(val) => {
                #[cfg(feature = "comments")]
                return Some(format!("{}<!-- {} -->", indent, val));
                #[cfg(not(feature = "comments"))]
                return None;
            }
            Self::Tag {
                decl,
                tag,
                attrs,
                children,
            } => {
                let all_str = if let Some(children) = children {
                    children.iter().all(|c| if let Type::Text = c.etype() {true} else {false})
                } else { true };
                Some(format!(
                    "{indent}<{}{}{}{}>{}{}",
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
                        let mut result = Vec::new();
                        let mut previous = Type::Other;

                        for (i, child) in children.iter().enumerate() {
                            let (lead, value) = if let Type::Other = child.etype() {
                                ("\n", child.display(offset+2))
                            } else if let Type::Other = previous {
                                ("\n", child.display(offset+2))
                            } else {
                                ("", child.display(0))
                            };

                            if let Some(value) = value {
                                previous = child.etype();
                                result.push(format!("{}{}", if i > 0 {lead} else {""}, value));
                            }
                        }

                        if all_str {
                            result.join("").trim().to_string()
                        } else {
                            format!("\n{}\n", result.join(""))
                        }
                    ),
                    or_new!((let Some(children) = children),
                        format!("{}</{}>", or_new!((children.len() > 0) => (!all_str), indent.clone()), tag)
                    ),
                    indent = indent
                ))
            },
        }
    }
}

impl Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(result) = self.debug(0) {
            write!(f, "{}", result)?
        }
        Ok(())
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = self.display(0) {
            write!(f, "{}", value)?
        }
        Ok(())
    }
}
