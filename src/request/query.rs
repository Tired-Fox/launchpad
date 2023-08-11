use crate::response::Result;
use serde::{Deserialize, Serialize};

pub trait IntoQuery {
    fn into_query(query: &str) -> Result<Query<Self>>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy)]
pub struct Query<T: IntoQuery>(pub T);

impl<T: IntoQuery> From<String> for Query<T> {
    fn from(value: String) -> Self {
        T::into_query(&value).unwrap()
    }
}

impl<'a, T: Deserialize<'a> + Default + Serialize> IntoQuery for T {
    fn into_query(query: &str) -> Result<Query<Self>>
    where
        Self: Sized,
    {
        let query = query.to_string();
        match serde_qs::from_str::<T>(Box::leak(query.clone().into_boxed_str())) {
            Ok(result) => Ok(Query(result)),
            Err(_) => match serde_plain::from_str::<T>(Box::leak(query.clone().into_boxed_str())) {
                Ok(result) => Ok(Query(result)),
                Err(_) => Err((
                    500,
                    format!(
                        "Failed to parse query from request; expected <span class=path>?{}</span>",
                        serde_qs::to_string(&T::default()).unwrap()
                    ),
                )),
            },
        }
    }
}
