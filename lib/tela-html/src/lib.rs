use std::{any::Any, collections::HashMap, fmt::Display};

pub mod element;
pub mod prelude;

pub use element::Element;
pub use proc::html;

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

    pub fn get(&self, key: &str) -> Option<&String> {
        self.props.get(key)
    }
}
