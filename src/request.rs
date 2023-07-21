use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, marker::PhantomData};

use crate::{Error, Result};

use super::response::JSON;

/// Placeholder state context
#[derive(Debug, Default)]
pub struct Empty;

/// A state/context manager used for individual endpoints
///
/// This is as a way of caching data per endpoint. The state can be of any
/// struct that implements `Debug + Default`.
///
/// # Example
/// ```rust
/// use launchpad::{prelude::*, State};
///
/// #[get]
/// fn hello_world(state: State<String>) -> Result<String> {
///     if state.inner() == "".to_string() {
///         state.inner_mut().push_str("Hello, world!");
///     }
/// }
/// ```
///
/// ```rust
/// use launchpad::{prelude::*, State};
///
/// #[derive(Debug)]
/// struct Example {
///     name: String,
///     count: usize
/// };
///
/// impl Default for Example {
///     fn default() -> Self {
///         Example {
///             name: "LaunchPad".to_string(),
///             count: 0
///         }
///     }
/// }
///
/// impl Example {
///     pub fn name(&self) -> &String {
///         &self.name
///     }
///     pub fn increment(&mut self) {
///         self.count += 1;
///     }
///     pub fn decrement(&mut self) {
///         if self.count > 0 {
///             self.count -= 1;
///         }
///     }
///     pub fn reset(&mut self) {
///         self.count = 0;
///     }
/// }
///
/// #[get]
/// fn hello_world(state: State<Example>) -> Result<String> {
///     if state.inner().count > 20 {
///         state.inner_mut().reset();
///     }
///
///     state.inner_mut().increment();
///     Ok(format!("{}: {}", state.inner().name(), state.inner().count))
/// }
/// ```
#[derive(Debug)]
pub struct State<T: Default + Debug>(T);
impl<T: Default + Debug> State<T> {
    pub fn get_ref(&self) -> &T {
        &self.0
    }

    pub fn get_ref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Default + Debug> Default for State<T> {
    fn default() -> Self {
        State(T::default())
    }
}

pub struct Plain(pub String);
impl<'de> Deserialize<'de> for Plain {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let input = String::deserialize(deserializer)?;
        Ok(Plain(input))
    }
}

/// A request content/body/data parser 
///
/// This object, given a struct, will parse the request body
/// into the provided struct that implements Default and serde::Deserialize.
///
/// # Example
/// ```rust
/// use launchpad::{prelude::*, Data};
///
/// #[derive(Default, Deserialize)]
/// struct ExampleData {
///     name: String
/// }
///
/// #[get]
/// fn hello_world(content: Data<ExampleData>) -> Result<String> {
///     return content.get_ref().name 
/// }
/// ```
pub struct Content<'a, T: Sized + Serialize + Deserialize<'a>>(T, PhantomData<&'a T>);

impl<'a, T:  Sized + Serialize + Deserialize<'a>> Content<'a, T> {
    pub fn parse(
        headers: &hyper::header::HeaderMap<hyper::header::HeaderValue>,
        body: &Bytes,
    ) -> Result<Content<'a, T>> {
        let data: &str = Box::leak(String::from_utf8(body.to_vec().clone()).unwrap().into_boxed_str());

        match headers.get("Content-Type") {
            Some(ctype) => {
                let ctype = ctype.to_str().unwrap().to_lowercase();
                if ctype.starts_with("application/json") {
                    JSON::<T>::parse(data).map(|json| json.0)
                } else if ctype.starts_with("text/plain") {
                    serde_plain::from_str::<T>(data).map_err(
                        |_| Error::new(500, "Failed to deserialize text/plain request body")
                    )
                } else {
                    Error::of(500, format!("Could not parse data from content type: {:?}", ctype))
                }
            }
            None => Error::of(500, "Unkown Content-Type: application/octet-stream"),
        }.map(|r| Content(r, PhantomData))
    }

    pub fn get_ref(&self) -> &T {
        &self.0
    }
}

/// A request query parser 
///
/// This object, given a struct, will parse the request url query 
/// into the provided struct that implements Default and serde::Deserialize.
///
/// # Example
/// ```rust
/// use launchpad::{prelude::*, Query};
///
/// #[derive(Default, Deserialize)]
/// struct ExampleQuery {
///     name: String
/// }
///
/// #[get]
/// fn hello_world(query: Query<ExampleQuery>) -> Result<String> {
///     return content.get_ref().name 
/// }
/// ```
pub struct Query<'a, T: Default + Deserialize<'a>>(T, PhantomData<&'a T>);

impl<'a, T: Default + Deserialize<'a>> Query<'a, T> {
    pub fn parse(
        uri: &'a hyper::Uri
    ) -> Result<Query<'a, T>> {
        match uri.query() {
            Some(query) => {
                match serde_qs::from_str::<T>(query) {
                    Ok(query) => Ok(Query(query, PhantomData)),
                    Err(error) => Error::of(500, format!("Failed to parse request query: {}", error))
                }
            },
            None => Ok(Query(T::default(), PhantomData))
        }
    }

    pub fn get_ref(&self) -> &T {
        &self.0
    }
}
