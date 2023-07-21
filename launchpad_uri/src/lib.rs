use std::{collections::HashMap, fmt::{Display, Debug}};

pub fn split<StrLike: Display>(uri: StrLike) -> Vec<String> {
    let mut uri = uri.to_string();
    if uri.starts_with("/") {
        uri = (&uri[1..]).to_string();
    }
    if uri.ends_with("/") {
        uri.pop();
    }
    uri.split("/").map(|s| s.to_string()).collect()
}

#[derive(Debug)]
pub enum Token {
    Segment(String),
    Capture(String, CType),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CType {
    String,
    Int,
    Path,
    Any,
}

#[derive(Debug, Clone)]
pub enum Prop {
    String(String),
    Int(i32),
    Path(String),
    Any(String),
}

impl Prop {
    pub fn parse(value: &String, ctype: &CType) -> Result<Self, ()> {
        match ctype {
            CType::String => Ok(Prop::String(value.clone())),
            CType::Any => Ok(Prop::Any(value.clone())),
            CType::Int => match value.parse::<i32>() {
                Ok(int) => Ok(Prop::Int(int)),
                _ => {
                    return Err(());
                }
            },
            CType::Path => {
                let mut value = value.strip_prefix("/").or(Some(value.as_str())).unwrap();
                value = value.strip_suffix("/").or(Some(value)).unwrap();
                Ok(Prop::Path(value.to_string()))
            }
        }
    }
}

impl Token {
    pub fn parse<StrLike: Display>(uri: &StrLike) -> Vec<Token> {
        split(uri)
            .iter()
            .map(|s| {
                if s.starts_with("<") && s.ends_with(">") {
                    Token::capture(s)
                } else {
                    Token::Segment(s.clone())
                }
            })
            .collect()
    }

    fn capture(segment: &String) -> Token {
        let segment = segment
            .strip_prefix("<")
            .unwrap()
            .strip_suffix(">")
            .unwrap()
            .trim();
        if segment.starts_with("...") {
            Token::Capture(segment[3..].to_string(), CType::Path)
        } else if segment.contains(":") {
            let parts = segment.split(":").collect::<Vec<&str>>();
            let name = parts[0].trim().to_string();
            let mut ctype = CType::String;
            if parts.len() > 1 {
                ctype = match parts[1].trim() {
                    "str" | "String" => CType::String,
                    "int" => CType::Int,
                    _ => panic!("Unkown uri capture type"),
                };
            }
            Token::Capture(name, ctype)
        } else {
            Token::Capture(segment.to_string(), CType::Any)
        }
    }

    pub fn segments<StrLike: Display>(uri: &StrLike) -> Vec<Token> {
        split(uri)
            .iter()
            .map(|s| Token::Segment(s.to_owned()))
            .collect()
    }
}
/// None means no match
/// Some(rank) means the uri works and this is the ranking
pub fn compare<S: Display, P: Display>(uri: &S, pattern: &P) -> Match {
    let uri = split(uri);
    let pattern = Token::parse(pattern);

    if pattern.len() == 0 {
        return Match::Discard;
    }

    let mut props: HashMap<String, Prop> = HashMap::new();
    let mut u = 0;
    let mut p = 0;
    let mut catch_all = false;

    while u < uri.len() && p < pattern.len() {
        match &pattern[p] {
            Token::Segment(pseg) => {
                if pseg == &uri[u] {
                    u += 1;
                    p += 1;
                } else {
                    return Match::Discard;
                }
            }
            Token::Capture(name, ctype) => {
                match ctype {
                    CType::Path => {
                        catch_all = true;
                        if p < pattern.len() - 1 {
                            p += 1;
                            if let Token::Segment(pseg) = &pattern[p] {
                                // iterate until segment found or return None
                                let start = u.clone();
                                match uri[start..].iter().position(|r| r == pseg) {
                                    Some(index) => {
                                        match Prop::parse(
                                            &(uri[start..start + index]).join("/"),
                                            &ctype,
                                        ) {
                                            Ok(prop) => {
                                                props.insert(name.clone(), prop);
                                                p += 1;
                                                u += index;
                                            }
                                            _ => return Match::Discard,
                                        }
                                    }
                                    None => return Match::Discard,
                                }
                            } else {
                                panic!("Expected path capture to have a normal segment following it")
                            }
                        } else {
                            match Prop::parse(&(uri[u..]).join("/"), &ctype) {
                                Ok(prop) => {
                                    props.insert(name.clone(), prop);
                                    p += 1;
                                    u += uri.len();
                                }
                                _ => return Match::Discard,
                            }
                        }
                    }
                    _ => match Prop::parse(&uri[u], &ctype) {
                        Ok(prop) => {
                            u += 1;
                            p += 1;
                            props.insert(name.clone(), prop);
                        }
                        _ => {
                            return Match::Discard;
                        }
                    },
                }
            }
        }
    }

    if (u == uri.len() && p < pattern.len()) || (p == pattern.len() && u < uri.len()) {
        Match::Discard
    } else {
        if catch_all {
            Match::Partial(pattern.len() as u8, props)
        } else {
            Match::Full(
                props
                    .values()
                    .into_iter()
                    .filter(|p| !matches!(p, Prop::Any(_)))
                    .count() as u8,
                props,
            )
        }
    }
}

pub fn props<S: Display, P: Display>(uri: &S, pattern: &P) -> HashMap<String, Prop> {
    match compare(&uri, &pattern) {
        Match::Full(_, props) => props,
        Match::Partial(_, props) => props,
        _ => HashMap::new()
    }
}

pub fn parse_props<P: Display>(pattern: &P) -> HashMap<String, CType> {
    let mut props = HashMap::new();
    for token in Token::parse(&pattern).iter() {
        match token {
            Token::Capture(name, ctype) => {props.insert(name.clone(), ctype.clone());},
            _ => ()
        };
    }
    props
}

#[derive(Debug)]
pub enum Match {
    Full(u8, HashMap<String, Prop>),
    Partial(u8, HashMap<String, Prop>),
    Discard,
}

impl From<Prop> for String {
    fn from(value: Prop) -> Self {
        match value {
            Prop::Any(v) | Prop::String(v) | Prop::Path(v) => v,
            _ => panic!("{:?} can not be converted to String", value)
        }
    }
}

impl From<Prop> for &str {
    fn from(value: Prop) -> Self {
        match value {
            Prop::Any(v) | Prop::String(v) | Prop::Path(v) => Box::leak(v.clone().into_boxed_str()),
            _ => panic!("{:?} can not be converted to String", value)
        }
    }
}

impl From<Prop> for i32 {
    fn from(value: Prop) -> Self {
        match value {
            Prop::Int(i) => i,
            _ => panic!("{:?} can not be converted to i32", value)
        }
    }
}

pub fn index(uri: &String, routes: &Vec<String>) -> Option<usize> {
    let mut ranks: Vec<(u8, usize)> = Vec::new();
    let mut full: Option<(u8, usize)> = None;
    for (i, pattern) in routes.iter().enumerate() {
        match compare(uri, pattern) {
            Match::Full(exact, _) => {
                match &full {
                    Some((e, _)) => {
                        if e < &exact {
                            full = Some((exact, i))
                        }
                    },
                    None => {
                        full = Some((exact, i))
                    }
                }
            }
            Match::Partial(rank, _) => {
                ranks.push((rank, i));
            }
            _ => ()
        }
    }
    ranks.sort_by(|f, s| (s.1 as u8).cmp(&(f.1 as u8)));

    match full {
        Some((_e, index)) => {
            Some(index)
        }
        None if ranks.len() > 0 => {
            Some(ranks[0].1)
        }
        _ => None,
    }

}

pub fn find<'a, T: Debug, StrLike: Display>(uri: &StrLike, routes: &'a Vec<T>, map: fn(&T) -> String) -> Option<&'a T> {
    index(&uri.to_string(), &routes.iter().map(map).collect::<Vec<String>>()).map(|index| &routes[index])
}

