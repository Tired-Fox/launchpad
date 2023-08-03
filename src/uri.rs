use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

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
    Capture(String),
    CatchAll(String),
}

impl Token {
    pub fn parse<StrLike: Display>(uri: &StrLike) -> Vec<Token> {
        split(uri)
            .iter()
            .map(|s| {
                if s.starts_with(":...") || s.starts_with(":") {
                    Token::capture(s)
                } else {
                    Token::Segment(s.clone())
                }
            })
            .collect()
    }

    fn capture(segment: &String) -> Token {
        if segment.starts_with(":...") {
            Token::CatchAll(segment[4..].to_string())
        } else if segment.starts_with(":") {
            Token::Capture(segment.strip_prefix(":").unwrap().to_string())
        } else {
            Token::Capture(segment.to_string())
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

    let mut props: HashMap<String, String> = HashMap::new();
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
            Token::Capture(name) => {
                props.insert(name.clone(), uri[u].to_string());
                u += 1;
                p += 1;
            }
            Token::CatchAll(name) => {
                catch_all = true;
                if p < pattern.len() - 1 {
                    p += 1;
                    if let Token::Segment(pseg) = &pattern[p] {
                        // iterate until segment found or return None
                        let start = u.clone();
                        match uri[start..].iter().position(|r| r == pseg) {
                            Some(index) => {
                                props.insert(name.clone(), uri[start..start + index].join("/"));
                                p += 1;
                                u += index;
                            }
                            None => return Match::Discard,
                        }
                    } else {
                        panic!("Expected path capture to have a normal segment following it")
                    }
                } else {
                    props.insert(name.clone(), (&uri[u..]).join("/"));
                    p += 1;
                    u += uri.len();
                }
            }
        }
    }

    if (u == uri.len() && p < pattern.len()) || (p == pattern.len() && u < uri.len()) {
        Match::Discard
    } else {
        let count = (pattern.len() - props.values().into_iter().count()) as u8;
        if catch_all {
            Match::Partial(count, props)
        } else {
            Match::Full(count, props)
        }
    }
}

pub fn props<S: Display, P: Display>(uri: &S, pattern: &P) -> HashMap<String, String> {
    match compare(&uri, &pattern) {
        Match::Full(_, props) => props,
        Match::Partial(_, props) => props,
        _ => HashMap::new(),
    }
}

pub fn parse_props<P: Display>(pattern: &P) -> Vec<String> {
    let mut props = Vec::new();
    for token in Token::parse(&pattern).iter() {
        match token {
            Token::Capture(name) | Token::CatchAll(name) => {
                props.push(name.clone());
            }
            _ => (),
        };
    }
    props
}

#[derive(Debug)]
pub enum Match {
    Full(u8, HashMap<String, String>),
    Partial(u8, HashMap<String, String>),
    Discard,
}

pub fn index(uri: &String, routes: &Vec<String>) -> Option<usize> {
    let mut ranks: Vec<(u8, usize)> = Vec::new();
    let mut full: Option<(u8, usize)> = None;
    for (i, pattern) in routes.iter().enumerate() {
        match compare(uri, pattern) {
            Match::Full(exact, _) => match &full {
                Some((e, _)) => {
                    if e < &exact {
                        full = Some((exact, i))
                    }
                }
                None => full = Some((exact, i)),
            },
            Match::Partial(rank, _) => {
                ranks.push((rank, i));
            }
            _ => (),
        }
    }
    ranks.sort_by(|f, s| (s.1 as u8).cmp(&(f.1 as u8)));

    match full {
        Some((_e, index)) => Some(index),
        None if ranks.len() > 0 => Some(ranks[0].1),
        _ => None,
    }
}

pub fn find<'a, StrLike: Display>(uri: &StrLike, routes: &'a Vec<String>) -> Option<String> {
    index(
        &uri.to_string(),
        &routes.iter().map(|m| m.to_string()).collect(),
    )
    .map(|index| (routes[index]).to_string())
}
