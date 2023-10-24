use std::{
    any, cmp::Ordering, collections::HashMap, fmt::Debug, path::PathBuf, str::FromStr, sync::Arc,
};

use async_trait::async_trait;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};

use crate::{
    error::Error,
    request::{FromRequest, FromRequestParts},
    response::StatusCode,
    server::Parts,
};

use super::handler::Handler;

lazy_static::lazy_static! {
    static ref MULTI_SLASH: regex::Regex = regex::Regex::new(r#"/+"#).unwrap();
    static ref WRAP_SLASH: regex::Regex = regex::Regex::new(r#"^/|/$"#).unwrap();
}

/// "/some/route/:path/nested"
/// "/some/route/:...path"
#[derive(Debug, PartialEq, Eq)]
pub enum PathToken<'a> {
    Segment(&'a str),
    Catch(&'a str),
    CatchAll(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Rank {
    Invalid(String),
    Match,
    Partial(u32),
}

impl Ord for Rank {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Rank::Match => match other {
                Rank::Match => Ordering::Equal,
                Rank::Invalid(_) => Ordering::Greater,
                Rank::Partial(_) => Ordering::Greater,
            },
            Rank::Invalid(_) => match other {
                Rank::Match => Ordering::Less,
                Rank::Invalid(_) => Ordering::Equal,
                Rank::Partial(_) => Ordering::Less,
            },
            Rank::Partial(own) => match other {
                Rank::Match => Ordering::Less,
                Rank::Invalid(_) => Ordering::Greater,
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
                Rank::Invalid(_) => Ordering::Greater,
                Rank::Partial(_) => Ordering::Greater,
            }),
            Rank::Invalid(_) => Some(match other {
                Rank::Match => Ordering::Less,
                Rank::Invalid(_) => Ordering::Equal,
                Rank::Partial(_) => Ordering::Less,
            }),
            Rank::Partial(own) => Some(match other {
                Rank::Match => Ordering::Less,
                Rank::Invalid(_) => Ordering::Greater,
                Rank::Partial(oth) => return own.partial_cmp(&oth),
            }),
        }
    }
}

/// Wrapper that represents the captures of a dynamic uri match.
#[derive(Debug, Clone, Default)]
pub struct Captures(Arc<HashMap<String, String>>);
impl Captures {
    pub fn new() -> Self {
        Captures(Arc::new(HashMap::new()))
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }

    pub fn get_as<T: FromStr>(&self, key: &str) -> Result<T, Error> {
        match self.0.get(key) {
            Some(value) => value.parse::<T>().map_err(|_| {
                Error::from((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to parse capture into {}", any::type_name::<T>()),
                ))
            }),
            None => Err(Error::from((StatusCode::INTERNAL_SERVER_ERROR, ""))),
        }
    }
}

impl<T> FromRequestParts<T> for Captures {
    fn from_request_parts(
        _request: &hyper::Request<Incoming>,
        parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(parts.catches.clone())
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> FromRequest<T> for Captures {
    async fn from_request(
        _request: hyper::Request<Incoming>,
        parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(parts.catches.clone())
    }
}

/// Represents the tokens that go into a dynamic path.
///
/// Has pre build pattern tokens that can be matched against another uri.
#[derive(Debug)]
pub struct RoutePath(&'static str, Vec<PathToken<'static>>);
impl RoutePath {
    pub fn path(&self) -> &str {
        &self.0
    }

    pub fn normalize(uri: &String) -> String {
        let uri = uri.trim().replace("\\", "/");
        let reduced_slash = MULTI_SLASH.replace_all(uri.as_str(), "/");
        WRAP_SLASH.replace_all(&reduced_slash, "").to_string()
    }

    pub fn new(uri: String) -> Self {
        let mut path = RoutePath(
            Box::leak(RoutePath::normalize(&uri).into_boxed_str()),
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

    pub fn compare(&self, uri: &str) -> (Rank, Captures) {
        let uri = RoutePath::normalize(&uri.to_string());
        if uri == self.0 {
            return (Rank::Match, Captures::new());
        }

        let uri = uri.split("/").collect::<Vec<&str>>();

        let mut catches = HashMap::new();
        let mut parts = uri.iter().peekable();
        let mut tokens = self.1.iter().peekable();
        let mut next_token = tokens.next();

        let mut rank = 0;
        loop {
            if let None = next_token {
                break;
            }

            if let None = parts.peek() {
                eprintln!("Not enough parts");
                return (
                    Rank::Invalid("Not enough parts to construct the uri pattern".to_string()),
                    Captures::new(),
                );
            }

            match next_token.unwrap() {
                PathToken::Segment(name) => {
                    let part = parts.next().unwrap();
                    if name != part {
                        return (
                            Rank::Invalid(format!(
                                "Segments do not match: {:?} != {:?}",
                                name, part
                            )),
                            Captures::new(),
                        );
                    }
                    rank += 1
                }
                PathToken::Catch(name) => {
                    let part = parts.next().unwrap();
                    catches.insert(name.to_string(), part.to_string());
                }
                PathToken::CatchAll(name) => {
                    let mut steps = 0;
                    let mut step_parts = 0;
                    let mut all_parts = true;

                    // Search for next concrete segment
                    for next in tokens.clone() {
                        match next {
                            PathToken::Segment(value) => {
                                all_parts = false;
                                let mut found_part = false;
                                for (i, part) in parts.clone().enumerate() {
                                    if part == value {
                                        found_part = true;
                                        step_parts = i;
                                        break;
                                    }
                                }
                                if !found_part {
                                    return (
                                        Rank::Invalid(format!("Uri pattern segment after catches not found in given uri: {:?}", value)),
                                        Captures::new(),
                                    );
                                }
                                break;
                            }
                            _ => steps += 1,
                        }
                    }

                    let to_take = if all_parts {
                        parts.clone().count()
                    } else {
                        step_parts - steps
                    };

                    catches.insert(
                        name.to_string(),
                        (0..to_take)
                            .filter_map(|_| match parts.next() {
                                Some(part) => Some(part.to_string()),
                                None => None,
                            })
                            .collect::<Vec<String>>()
                            .join("/"),
                    );
                }
            };

            next_token = tokens.next();
        }

        (Rank::Partial(rank), Captures(Arc::new(catches)))
    }
}

/// Wrapper around a route handler pointer.
#[derive(Clone)]
pub struct BoxedHandler<I, S: Send + Sync + 'static = ()>(Arc<dyn Handler<I, S>>);

impl<I, S> BoxedHandler<I, S>
where
    I: Send + Sync + 'static,
    S: Send + Sync + 'static,
{
    pub fn from_handler<H>(handler: H) -> Self
    where
        H: Handler<I, S>,
    {
        BoxedHandler(Arc::new(handler))
    }

    pub async fn call(
        &self,
        request: hyper::Request<Incoming>,
        state: Option<S>,
        catches: Captures,
    ) -> hyper::Response<Full<Bytes>> {
        (self.0).handle(request, state, catches).await
    }
}

/// Allows the dynamic route handler pointer to be called.
#[async_trait]
pub trait ErasedHandler<S: Send + Sync + 'static>: Send + Sync + 'static {
    async fn call(
        &self,
        request: hyper::Request<Incoming>,
        state: Option<S>,
        catches: Captures,
    ) -> hyper::Response<Full<Bytes>>;
}

#[async_trait]
impl<I, S> ErasedHandler<S> for BoxedHandler<I, S>
where
    I: Send + Sync + 'static,
    S: Send + Sync + 'static,
{
    async fn call(
        &self,
        request: hyper::Request<Incoming>,
        state: Option<S>,
        catches: Captures,
    ) -> hyper::Response<Full<Bytes>> {
        self.call(request, state, catches).await
    }
}

/// Wrapper around a route handler.
#[derive(Clone)]
pub struct Endpoint<S: Send + Sync + 'static>(pub Arc<dyn ErasedHandler<S>>);
impl<S> Endpoint<S>
where
    S: Send + Sync + 'static,
{
    pub fn new<E: ErasedHandler<S>>(handler: E) -> Self {
        Endpoint(Arc::new(handler))
    }
}

impl<S: Send + Sync + 'static> Debug for Endpoint<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Endpoint",)
    }
}

/// A wrapper that holds handlers for a given route.
#[derive(Debug)]
pub struct Route<S: Send + Sync + 'static> {
    callbacks: RouteMethods<S>,
}

impl<S: Send + Sync + 'static> Route<S> {
    fn replace_or_not(endpoint: &mut Option<Endpoint<S>>, new: Option<Endpoint<S>>) {
        if let Some(_) = &new {
            *endpoint = new
        }
    }
}

/// A wrapper arround a mapping routes to their handlers.
#[derive(Debug)]
pub struct Routes<S: Send + Sync + 'static> {
    paths: Vec<(RoutePath, Route<S>)>,
    cache: HashMap<String, (usize, Captures)>,
}

impl<S: Send + Sync + 'static> Routes<S> {
    pub fn insert(&mut self, key: String, value: Route<S>) {
        match self
            .paths
            .iter_mut()
            .find(|val| val.0.path() == RoutePath::normalize(&key))
        {
            Some((_, route)) => route.merge(value),
            None => self.paths.push((RoutePath::new(key), value)),
        }
    }

    pub fn fetch(&mut self, uri: &str, method: &hyper::Method) -> Option<(&Endpoint<S>, Captures)> {
        let key = RoutePath::normalize(&uri.to_string());
        if self.cache.contains_key(&key) {
            let (index, catches) = self.cache.get(&key).unwrap();
            let (_, route) = self.paths.get(*index).unwrap();
            route.fetch(method).map(|v| (v, catches.clone()))
        } else {
            let mut partials = Vec::new();
            for (i, (path, route)) in self.paths.iter().enumerate() {
                let (rank, catches) = path.compare(uri);
                match rank {
                    Rank::Match => {
                        // TODO: Add to cache and return this option
                        self.cache.insert(key.clone(), (i, catches));
                        let (_, catches) = self.cache.get(&key).unwrap();
                        return route.fetch(method).map(|v| (v, catches.clone()));
                    }
                    Rank::Invalid(_) => { /* Ignore for now */ }
                    Rank::Partial(_) => partials.push((i, rank, catches)),
                }
            }

            partials.sort_by(|a, b| a.0.cmp(&b.0));
            if let Some(partial) = partials.last() {
                self.paths
                    .get(partial.0)
                    .unwrap()
                    .1
                    .fetch(method)
                    .map(|v| (v, partial.2.clone()))
            } else {
                None
            }
        }
    }

    pub fn new() -> Self {
        Routes {
            paths: Vec::new(),
            cache: HashMap::new(),
        }
    }
}

#[doc = "Create a new route with the handler that handles any request method"]
pub fn any<H, T, S: Send + Sync + 'static>(handler: H) -> Route<S>
where
    H: Handler<T, S>,
    T: Send + Sync + 'static,
{
    Route {
        callbacks: RouteMethods {
            any: Some(Endpoint(Arc::new(BoxedHandler::from_handler(handler)))),
            ..Default::default()
        },
    }
}

macro_rules! make_methods {
    ($($method: ident),*) => {
        paste::paste! {
            $(
                #[doc="Create a new route with the " $method " method handler"]
                pub fn [<$method:lower>]<H, T, S>(callback: H) -> crate::server::router::route::Route<S>
                where
                    H: Handler<T, S>,
                    T: Send + Sync + 'static,
                    S: Send + Sync + 'static,
                {
                    crate::server::router::route::Route {
                        callbacks: crate::server::router::route::RouteMethods {
                            [<$method:lower>]: Some(crate::server::router::route::Endpoint(Arc::new(BoxedHandler::from_handler(callback)))),
                            ..Default::default()
                        },
                    }
                }
            )*
        }
        paste::paste! {
            impl<S: Send + Sync + 'static> Route<S> {
                /// Merge duplicate route paths together. New handlers override old handlers.
                fn merge(&mut self, new: Route<S>) {
                    $(Route::replace_or_not(&mut self.callbacks.[<$method:lower>], new.callbacks.[<$method:lower>]);)*
                }

                pub fn fetch(&self, method: &hyper::Method) -> Option<&Endpoint<S>> {
                    use hyper::Method;
                    match method {
                        $(&Method::$method => match &self.callbacks.[<$method:lower>]{
                            // If endpoint doesn't exist use fallback
                            None => self.callbacks.any.as_ref(),
                            Some(valid) => Some(valid)
                        },)*
                        _ => None,
                    }
                }

                #[doc="Any method handler"]
                pub fn any<H, T>(mut self, handler: H) -> Self
                where
                    H: Handler<T, S>,
                    T: Send + Sync + 'static,
                {
                    self.callbacks.any =
                        Some($crate::server::router::route::Endpoint(Arc::new(BoxedHandler::from_handler(handler))));
                    self
                }

                $(
                    #[doc=$method " method handler"]
                    pub fn [<$method:lower>]<H, T>(mut self, handler: H) -> Self
                    where
                        H: Handler<T, S>,
                        T: Send + Sync + 'static,
                    {
                        self.callbacks.[<$method:lower>] =
                            Some($crate::server::router::route::Endpoint(Arc::new(BoxedHandler::from_handler(handler))));
                        self
                    }
                )*
            }
        }
        paste::paste! {
            /// All method handlers for a given route.
            #[derive(Debug)]
            pub struct RouteMethods<S: Send + Sync + 'static> {
                $([<$method:lower>]: Option<Endpoint<S>>,)*
                any: Option<Endpoint<S>>,
            }

            impl<S: Send + Sync + 'static> Default for RouteMethods<S> {
                fn default() -> Self {
                    RouteMethods {
                        any: None,
                        $([<$method:lower>]: None,)*
                    }
                }
            }
        }
    };
}

make_methods! {GET, POST, DELETE, PUT, HEAD, CONNECT, OPTIONS, TRACE, PATCH}

/// Convert a path into a uri path starting with `/`.
///
/// # Example
/// ```
/// "some/path\\here" -> "/some/path/here"
/// ```
pub(crate) fn to_uri(uri: &String) -> String {
    let mut uri = uri.replace("\\", "/").replace("//", "/");
    if !uri.starts_with("/") {
        uri = String::from("/") + uri.as_str();
    }
    uri
}

pub trait IntoStaticPath {
    fn into_static_path(self) -> (String, PathBuf);
}

impl IntoStaticPath for String {
    fn into_static_path(self) -> (String, PathBuf) {
        let uri = to_uri(&self);
        (uri, PathBuf::from(self))
    }
}

impl IntoStaticPath for &str {
    fn into_static_path(self) -> (String, PathBuf) {
        let uri = to_uri(&self.to_string());
        (uri, PathBuf::from(self))
    }
}

impl<S1: ToString, S2: ToString> IntoStaticPath for (S1, S2) {
    fn into_static_path(self) -> (String, PathBuf) {
        let uri = to_uri(&self.0.to_string());
        (uri, PathBuf::from(self.1.to_string()))
    }
}
