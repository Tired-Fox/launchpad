use std::{cmp::Ordering, collections::HashMap};

use tela::prelude::Error;

lazy_static::lazy_static! {
    static ref MULTI_SLASH: regex::Regex = regex::Regex::new(r#"/+"#).unwrap();
    static ref WRAP_SLASH: regex::Regex = regex::Regex::new(r#"^/|/$"#).unwrap();
}

/// "/some/route/:path/nested"
/// "/some/route/:...path"
#[derive(Debug, PartialEq, Eq)]
enum PathToken<'a> {
    Segment(&'a str),
    Catch(&'a str),
    CatchAll(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
enum Rank {
    Invalid,
    Match,
    Partial(u32),
}

impl Ord for Rank {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Rank::Match => match other {
                Rank::Match => Ordering::Equal,
                Rank::Invalid => Ordering::Greater,
                Rank::Partial(_) => Ordering::Greater,
            },
            Rank::Invalid => match other {
                Rank::Match => Ordering::Less,
                Rank::Invalid => Ordering::Equal,
                Rank::Partial(_) => Ordering::Less,
            },
            Rank::Partial(own) => match other {
                Rank::Match => Ordering::Less,
                Rank::Invalid => Ordering::Greater,
                Rank::Partial(oth) => return own.cmp(&oth),
            },
        }
    }
}

impl PartialOrd for Rank {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            Rank::Match => Some(match other {
                Rank::Match => Ordering::Equal,
                Rank::Invalid => Ordering::Greater,
                Rank::Partial(_) => Ordering::Greater,
            }),
            Rank::Invalid => Some(match other {
                Rank::Match => Ordering::Less,
                Rank::Invalid => Ordering::Equal,
                Rank::Partial(_) => Ordering::Less,
            }),
            Rank::Partial(own) => Some(match other {
                Rank::Match => Ordering::Less,
                Rank::Invalid => Ordering::Greater,
                Rank::Partial(oth) => return own.partial_cmp(&oth),
            }),
        }
    }
}

#[derive(Debug)]
struct Catches(HashMap<String, String>);
impl Catches {
    pub fn new() -> Self {
        Catches(HashMap::new())
    }
}

#[derive(Debug)]
struct RoutePath<'a>(&'a str, Vec<PathToken<'a>>);
impl<'a> RoutePath<'a> {
    fn normalize(uri: String) -> String {
        let uri = uri.trim().replace("\\", "/");
        let reduced_slash = MULTI_SLASH.replace_all(uri.as_str(), "/");
        WRAP_SLASH.replace_all(&reduced_slash, "").to_string()
    }
    fn new(uri: String) -> Self {
        let mut path = RoutePath(
            Box::leak(RoutePath::normalize(uri).into_boxed_str()),
            Vec::new(),
        );

        for segment in path.0.split("/") {
            if segment.starts_with(":") {
                if segment.starts_with(":...") {
                    path.1.push(PathToken::CatchAll(&segment[4..]));
                } else {
                    path.1.push(PathToken::Catch(&segment[1..]));
                }
            } else {
                path.1.push(PathToken::Segment(segment))
            }
        }

        path
    }

    pub fn compare(&self, uri: &str) -> (Rank, Catches) {
        if uri == self.0 {
            return (Rank::Match, Catches::new());
        }

        let uri = RoutePath::normalize(uri.to_string());
        let uri = uri.split("/").collect::<Vec<&str>>();

        let mut catches = Catches(HashMap::new());
        let mut parts = uri.iter();
        let mut tokens = self.1.iter();
        let mut next_token = tokens.next();

        let mut rank = 0;
        loop {
            if let None = next_token {
                break;
            }

            let part = if let Some(np) = parts.next() {
                np
            } else {
                eprintln!("Not enough parts");
                return (Rank::Invalid, Catches::new());
            };

            match next_token.unwrap() {
                PathToken::Segment(name) => {
                    if name != part {
                        eprintln!("Segments don't match: {:?} == {:?}", name, part);
                        return (Rank::Invalid, Catches::new());
                    }
                    rank += 1
                }
                PathToken::Catch(name) => {
                    catches.0.insert(name.to_string(), part.to_string());
                }
                PathToken::CatchAll(name) => {
                    // TODO: Find next segment or start from end and catch backwards with remaining
                    // being the catch all
                    let cai = tokens.clone();
                    let pai = parts.clone();
                    // TODO: Remove this early return
                    return (Rank::Partial(rank), catches);
                }
            };

            next_token = tokens.next();
        }
        println!("{}", rank);

        (Rank::Partial(rank), catches)
    }
}

// async fn handler() -> html::Element {
//     html::new!(<h1>"Hello, world!"</h1>)
// }

fn main() {
    let route = RoutePath::new("///\\/blog/:year\\//:month\\:day/:...sub/:sub".to_string());
    println!(
        "{:?}",
        route.compare("/blog/2023/10/20/somewhere/over/the/rainbow")
    );
    // Server::builder()
    //     .on_bind(|addr| println!("{}", addr))
    //     .serve(
    //         Socket::Local(3000),
    //         Router::builder().route("/", get(handler)),
    //     )
    //     .await;
}
