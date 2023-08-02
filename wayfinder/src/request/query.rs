use crate::response::Result;
use hyper::Uri;
use serde::Deserialize;

pub trait IntoQuery {
    fn into_query(query: &str) -> Result<Query<Self>>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy)]
pub struct Query<T: IntoQuery>(pub T);
impl<T: IntoQuery> Query<T> {
    pub fn extract(uri: &mut Uri) -> Result<Self>
    where
        Self: Sized,
    {
        match uri.query() {
            Some(query) => T::into_query(query),
            _ => Err((500, "No query to parse".to_string())),
        }
    }
}

impl<'a, T: Deserialize<'a>> IntoQuery for T {
    fn into_query(query: &str) -> Result<Query<Self>>
    where
        Self: Sized,
    {
        let query = query.to_string();
        match serde_qs::from_str::<T>(Box::leak(query.clone().into_boxed_str())) {
            Ok(result) => Ok(Query(result)),
            Err(_) => match serde_plain::from_str::<T>(Box::leak(query.into_boxed_str())) {
                Ok(result) => Ok(Query(result)),
                Err(_) => Err((500, "Failed to parse query from request".to_string())),
            },
        }
    }
}
