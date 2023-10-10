/*
 *  <div item=3>
 *      {some_var}
 *  </div>
 */

use std::{collections::HashMap, fmt::Display};

// use tela_html::components::*
pub use proc::html;

pub trait ToAttrValue {
    fn to_attr_value(&self) -> Option<String>;
}

pub trait Component<T, R> {
    fn create_component(&self, props: T) -> R;
}

impl<F> Component<Props, String> for F
where
    F: Fn(Props) -> String,
{
    fn create_component(&self, props: Props) -> String {
        self(props)
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

pub trait ToSpread {
    fn to_spread(self) -> HashMap<String, String>;
}

impl<A: Display, B: Display, I: IntoIterator<Item = (A, B)>> ToSpread for I {
    fn to_spread(self) -> HashMap<String, String> {
        self.into_iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect()
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
    children: Vec<String>,
    captures: Vec<String>,
}

impl Props {
    pub fn attributes(&self) -> String {
        let mut result = self.captures.clone();
        let props = self
            .props
            .iter()
            .filter_map(|(name, value)| value.to_attr_value().map(|v| format!("{}{}", name, v)))
            .collect::<Vec<String>>()
            .join(" ");
        if !props.is_empty() {
            result.push(props)
        }
        result.join(" ")
    }

    pub fn children(&self) -> &Vec<String> {
        &self.children
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.props.get(key)
    }
}

impl<T: ToSpread, const S1: usize, const S3: usize, const S4: usize>
    From<(
        [(&str, Box<dyn Display>); S1],
        [Box<dyn Display>; S3],
        [String; S4],
        Option<T>,
    )> for Props
{
    fn from(
        value: (
            [(&str, Box<dyn Display>); S1],
            [Box<dyn Display>; S3],
            [String; S4],
            Option<T>,
        ),
    ) -> Self {
        let mut props = Props::default();
        for (name, value) in value.0 {
            props.props.insert(name.to_string(), value.to_string());
        }
        for capture in value.1 {
            props.captures.push(capture.to_string())
        }
        props.children.extend_from_slice(&value.2);
        match value.3 {
            None => {}
            Some(val) => {
                let value = val.to_spread();
                for (name, value) in value.iter() {
                    if name.starts_with(":") {
                        props
                            .props
                            .insert((&name[1..]).to_string(), value.to_string());
                    } else {
                        props.props.insert(name.to_string(), value.to_string());
                    }
                }
            }
        }
        props
    }
}
