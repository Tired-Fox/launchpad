use bytes::Bytes;
use serde::Deserialize;
use std::{fmt::Debug, marker::PhantomData};

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
/// fn hello_world(state: State<String>) -> String {
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
    pub fn inner(&self) -> &T {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Default + Debug> Default for State<T> {
    fn default() -> Self {
        State(T::default())
    }
}

/// Placeholder state context
#[derive(Debug, Default)]
pub struct Empty;

pub struct Data<'a, B: Default + Deserialize<'a>>(B, PhantomData<&'a B>);

impl<'a, T: Default + Deserialize<'a>> Data<'a, T> {
    pub fn parse(
        headers: &hyper::header::HeaderMap<hyper::header::HeaderValue>,
        body: &Bytes,
    ) -> Result<Data<'a, T>, (u16, String)> {
        match headers.get("Content-Type") {
            Some(ctype) => {
                let ctype = ctype.to_str().unwrap().to_lowercase();
                if ctype.starts_with("application/json") {
                    parse_json(body)
                } else {
                    Err((
                        500,
                        format!("Could not parse data from content type: {:?}", ctype),
                    ))
                }
            }
            None => Ok(Data(T::default(), PhantomData)),
        }
    }

    pub fn get_ref(&self) -> &T {
        &self.0
    }
}

fn parse_json<'a, T: Default + Deserialize<'a>>(
    body: &Bytes,
) -> Result<Data<'a, T>, (u16, String)> {
    let data: String = String::from_utf8(body.to_vec().clone()).unwrap();

    let result: T = match serde_json::from_str::<T>(Box::leak(data.into_boxed_str())) {
        Ok(res) => res,
        Err(_) => return Err((500, "Failed to parse json from request".to_string())),
    };

    Ok(Data(result, PhantomData))
}
