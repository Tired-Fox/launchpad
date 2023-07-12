use std::fmt::{Debug, Display};
use crate::Response;

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

pub struct Data<T: Default>(T);
impl<T: Default> Data<T> {
    pub fn parse(_request: &hyper::Request<hyper::body::Incoming>) -> Result<Data<T>, (u16, String)> {
        // Ok(Data(T::default()));
        Err((500, "Request data parsing is not implemented".to_string()))
    }

    pub fn get_ref(&self) -> &T {
        &self.0
    }
}
