use std::{collections::HashMap, fmt::Display, hash::Hash};

pub mod element;
pub mod prelude;

pub use element::Element;
pub use proc::{html, Prop};
use serde::{Deserialize, Serialize};

macro_rules! impl_prop {
    ([$($name: ty),* $(,)?]) => {
        $(
            impl ToProp for $name {
                 fn to_prop(&self) -> String {
                     self.to_string()
                 }
            }

            impl FromProp for $name {
                fn from_prop(prop: String) -> Result<Self, String> {
                    prop.parse::<$name>().map_err(|e| e.to_string())
                }
            }
        )*
    };
}

pub trait FromProp
where
    Self: Sized,
{
    fn from_prop(prop: String) -> Result<Self, String>;
}

pub trait FromProps<const SIZE: usize>
where
    Self: Sized,
{
    fn from_props(extracts: [&str; SIZE], props: &HashMap<String, String>) -> Result<Self, String>;
}

macro_rules! multi_prop_extract {
    ($([$($set: tt)*]);* $(;)?) => {
        $(
            multi_prop_extract!{ $($set)* }
        )*
    };
    ($first: literal) => {
        paste::paste!{
            impl<[<T $first>]: FromProp> FromProps<$first> for ([<T $first>],) {
                fn from_props(extracts: [&str; $first], props: &HashMap<String, String>) -> Result<Self, String> {
                    Ok((
                        [<T $first>]::from_prop(match props.get(&(extracts[$first - 1].to_string())) {
                            Some(value) => value,
                            None => return Err(format!("Key '{}' is not a prop", extracts[$first-1]))
                        }.clone())?,
                    ))
                }
            }
        }
    };
    ($($type: literal),* | $last: literal) => {
        paste::paste!{
            impl<$([<T $type>]: FromProp,)* [<T $last>]: FromProp> FromProps<$last> for ($([<T $type>],)* [<T $last>], ) {
                fn from_props(extracts: [&str; $last], props: &HashMap<String, String>) -> Result<Self, String> {
                    Ok((
                        $(
                            [<T $type>]::from_prop(match props.get(&(extracts[$type - 1].to_string())) {
                                Some(value) => value,
                                None => return Err(format!("Key '{}' is not a prop", extracts[$type-1]))
                            }.clone())?,
                        )*
                        [<T $last>]::from_prop(match props.get(&(extracts[$last - 1].to_string())) {
                            Some(value) => value,
                            None => return Err(format!("Key {} is not a prop", extracts[$last-1]))
                        }.clone())?,
                    ))
                }
            }
        }
    };
}

multi_prop_extract! {
    [1];
    [1 | 2];
    [1, 2 | 3];
    [1, 2, 3 | 4];
    [1, 2, 3, 4 | 5];
    [1, 2, 3, 4, 5 | 6];
    [1, 2, 3, 4, 5, 6 | 7];
    [1, 2, 3, 4, 5, 6, 7 | 8];
    [1, 2, 3, 4, 5, 6, 7, 8 | 9];
    [1, 2, 3, 4, 5, 6, 7, 8, 9 | 10];
    [1, 2, 3, 4, 5, 6, 7, 8, 9, 10 | 11];
    [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 | 12];
    [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12 | 13];
    [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13 | 14];
    [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14 | 15];
}

impl<T: Prop> FromProp for T {
    fn from_prop(prop: String) -> Result<Self, String> {
        serde_json::from_str(Box::leak(prop.into_boxed_str())).map_err(|e| e.to_string())
    }
}

pub trait ToProp
where
    Self: Sized,
{
    fn to_prop(&self) -> String;
}

impl<T: Prop> ToProp for T {
    fn to_prop(&self) -> String {
        match serde_json::to_string(self) {
            Ok(value) => value,
            Err(e) => e.to_string(),
        }
    }
}

impl FromProp for String {
    fn from_prop(prop: String) -> Result<Self, String> {
        Ok(prop.clone())
    }
}

impl ToProp for String {
    fn to_prop(&self) -> String {
        self.clone()
    }
}

impl FromProp for &'static str {
    fn from_prop(prop: String) -> Result<Self, String> {
        Ok(Box::leak(prop.into_boxed_str()))
    }
}

impl ToProp for &str {
    fn to_prop(&self) -> String {
        self.to_string()
    }
}

impl_prop!([i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool]);

pub trait Prop: Serialize + Deserialize<'static> {}

impl<A: Serialize + Deserialize<'static> + Hash + Eq, B: Serialize + Deserialize<'static>> Prop
    for HashMap<A, B>
{
}

impl<T: Serialize + Deserialize<'static>> Prop for Vec<T> {}

#[macro_export]
macro_rules! props {
    ($($key: ident: $value: expr),* $(,)?) => {
        [$((stringify!($key).replace("_", "-"),$value.to_string()),)*]
    };
}

pub trait ToAttrValue {
    fn to_attr_value(&self) -> Option<String>;
}

pub trait Component {
    fn create_component(
        &self,
        attributes: HashMap<String, String>,
        children: Vec<Element>,
    ) -> Element;
}

impl<F> Component for F
where
    F: Fn(Props) -> Element,
{
    fn create_component(
        &self,
        attributes: HashMap<String, String>,
        children: Vec<Element>,
    ) -> Element {
        // let callback: fn(dyn Any) -> Element = |v| Element::None;
        self(Props::new(attributes, children))
    }
}

impl<T> ToAttrValue for T
where
    T: Display,
{
    fn to_attr_value(&self) -> Option<String> {
        let value = self.to_string();
        if ["yes", "true"].contains(&value.as_str()) {
            Some(String::new())
        } else if ["no", "false"].contains(&value.as_str()) {
            None
        } else {
            Some(format!("={:?}", self.to_string()))
        }
    }
}

pub trait ToAttributes {
    fn to_attributes(self) -> String;
}

impl<A: Display, B: Display, I: IntoIterator<Item = (A, B)>> ToAttributes for I {
    fn to_attributes(self) -> String {
        self.into_iter()
            .filter_map(|(name, value)| value.to_attr_value().map(|v| format!("{}{}", name, v)))
            .collect::<Vec<String>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, Default)]
pub struct Props {
    props: HashMap<String, String>,
    children: Vec<Element>,
}

impl Props {
    pub fn new(props: HashMap<String, String>, children: Vec<Element>) -> Self {
        Props { props, children }
    }

    pub fn children(&self) -> &Vec<Element> {
        &self.children
    }

    pub fn fetch<T: FromProp>(&self, key: &str) -> Result<T, String> {
        match self.props.get(key) {
            Some(value) => T::from_prop(value.clone()),
            None => Err(format!("Key not found in props: {}", key)),
        }
    }

    pub fn fetch_all<const SIZE: usize, T: FromProps<SIZE>>(
        &self,
        keys: [&str; SIZE],
    ) -> Result<T, String> {
        T::from_props(keys, &self.props)
    }

    pub fn get(&self, key: &str) -> Option<String> {
        #[cfg(feature="auto-escape")]
        return self.props.get(key).map(|v| unescape(v.clone()));
        #[cfg(not(feature="auto-escape"))]
        return self.props.get(key).map(|v| v.clone());
    }
}

/// HTML escape the provided string
pub fn escape(value: String) -> String {
    #[cfg(not(feature = "auto-escape"))]
    return value;
    #[cfg(feature = "auto-escape")]
    {
        let mut result = String::new();
        for char in value.chars() {
            match char {
                '&' => result.push_str("&amp;"),
                '<' => result.push_str("&lt;"),
                '>' => result.push_str("&gt;"),
                '"' => result.push_str("&quot;"),
                '\'' => result.push_str("&#x27;"),
                _ => result.push(char),
            }
        }
        result
    }
}

/// HTML unescape the provided string
pub fn unescape(value: String) -> String {
    #[cfg(not(feature = "auto-escape"))]
    return value;
    #[cfg(feature = "auto-escape")]
    {
        value
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&lt;", "<")
            .replace("&#x27;", "'")
            .replace("&amp;", "&")
    }
}
