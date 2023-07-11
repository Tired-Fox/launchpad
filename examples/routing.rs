//! /api/name/<first>/<last>
//! /api/name/zachary/boehm
//! /api/name props: {first: 'zachary', last: 'boehm'}
//!
//! /api/<...path>
//! /api/some/path/here
//! /api props: {path: 'some/path/here'}

// (1) /api/name/<first>/<last: int> <- If last is int
// (2) /api/name/<first>/<last> <- yes
// (3) /api/<...path> <- No because it doesn't have the name segment while others do
//
// /api/name/zachary/boehm (2)
// /api/name/zachary/32    (1)
// /api/some/other/path    (3)
//
// Order endpoints by most matching/valid segments
// run through all endpoints that match from most matching segments to least
// then return the first one that is valid when parsing

use std::{collections::HashMap, sync::Arc};

mod uri {
    use std::{collections::HashMap, fmt::Display};

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

    #[derive(Debug, PartialEq, Eq)]
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

    #[derive(Debug)]
    pub enum Match {
        Full(u8, HashMap<String, Prop>),
        Partial(u8, HashMap<String, Prop>),
        Discard,
    }

    pub fn index(uri: &String, routes: &Vec<String>) -> Option<(usize, HashMap<String, Prop>)> {
        let mut ranks: Vec<(u8, HashMap<String, Prop>, usize)> = Vec::new();
        let mut full: Option<(u8, HashMap<String, Prop>, usize)> = None;
        for (i, pattern) in routes.iter().enumerate() {
            match compare(uri, pattern) {
                Match::Full(exact, props) => {
                    match &full {
                        Some((e, _, _)) => {
                            if e < &exact {
                                full = Some((exact, props, i))
                            }
                        },
                        None => {
                            full = Some((exact, props, i))
                        }
                    }
                }
                Match::Partial(rank, props) => {
                    ranks.push((rank, props, i));
                }
                _ => ()
            }
        }
        ranks.sort_by(|f, s| (s.2 as u8).cmp(&(f.2 as u8)));

        match full {
            Some((_e, props, endpoint)) => {
                Some((endpoint, props))
            }
            None if ranks.len() > 0 => {
                Some((ranks[0].2, ranks[0].1.clone()))
            }
            _ => None,
        }

    }

    pub fn find<'a, T, StrLike: Display>(uri: &StrLike, routes: &'a Vec<T>, map: fn(&T) -> String) -> Option<(&'a T, HashMap<String, Prop>)> {
        index(&uri.to_string(), &routes.iter().map(map).collect::<Vec<String>>()).map(|(index, props)| (&routes[index], props))
    }
}

extern crate launchpad;
use launchpad::prelude::*;

fn main() {
    let puri = "/api/name/<first>/<last>";
    let ptyped_uri = "/api/name/<first: str>/<age: int>";
    let ppath_uri = "/api/<...path>";

    let ruri = "/api/name/zachary/boehm";
    let rtyped_uri = "/api/name/zachary/32";
    let rpath_uri = "/api/some/path/here";

    println!("\n\x1b[36mPatterns:\n\x1b[39m:\x1b[33muri\x1b[39m: {:?}\n\x1b[33mtyped uri\x1b[39m: {:?}\n\x1b[33mpath uri\x1b[39m: {:?}",
        uri::Token::parse(&puri),
        uri::Token::parse(&ptyped_uri),
        uri::Token::parse(&ppath_uri)
    );

    println!("\n\x1b[36mRoute\x1b[39m:\n\x1b[33muri\x1b[39m: {:?}\n\x1b[33mtyped uri\x1b[39m: {:?}\n\x1b[33mpath uri\x1b[39m: {:?}",
        ruri,
        rtyped_uri,
        rpath_uri
    );

    let routes = vec![puri, ptyped_uri, ppath_uri];
    for uri in [ruri, rtyped_uri, rpath_uri].iter() {
        match uri::find::<&str, _>(uri, &routes, |m| m.to_string()) {
            Some((endpoint, props)) => {
                println!("{:?} -> {:?} @{:?}", endpoint, uri, props);
            },
            None => println!("No endpoint for {:?}", uri) 
        }
    }
}
