use std::fmt::Debug;

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

#[derive(Debug, Default)]
pub struct Empty;
