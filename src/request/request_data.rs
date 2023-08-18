use crate::response::Result;

use super::{body::IntoBody, query::IntoQuery, Body, Query};

pub trait ToParam<T> {
    fn to_param(&mut self) -> Result<T>;
}
pub struct RequestData(pub hyper::Uri, pub hyper::Method, pub Vec<u8>);

impl<T: IntoQuery> ToParam<Query<T>> for RequestData {
    fn to_param(&mut self) -> Result<Query<T>> {
        match self.0.query() {
            Some(query) => T::into_query(query),
            _ => Err((500, "No query to parse".to_string())),
        }
    }
}

impl<T: IntoQuery> ToParam<Option<Query<T>>> for RequestData {
    fn to_param(&mut self) -> Result<Option<Query<T>>> {
        match self.0.query() {
            Some(query) => Ok(T::into_query(query).ok()),
            _ => Ok(None),
        }
    }
}

impl<T: IntoQuery> ToParam<Result<Query<T>>> for RequestData {
    fn to_param(&mut self) -> Result<Result<Query<T>>> {
        match self.0.query() {
            Some(query) => Ok(T::into_query(query)),
            _ => Ok(Err((500, "No query to parse".to_string()))),
        }
    }
}

impl<T: IntoBody> ToParam<Body<T>> for RequestData {
    fn to_param(&mut self) -> Result<Body<T>> {
        let body = std::str::from_utf8(&self.2[..]).unwrap();
        T::into_body(body)
    }
}

impl<T: IntoBody> ToParam<Option<Body<T>>> for RequestData {
    fn to_param(&mut self) -> Result<Option<Body<T>>> {
        let body = std::str::from_utf8(&self.2[..]).unwrap();
        Ok(T::into_body(body).ok())
    }
}

impl<T: IntoBody> ToParam<Result<Body<T>>> for RequestData {
    fn to_param(&mut self) -> Result<Result<Body<T>>> {
        let body = std::str::from_utf8(&self.2[..]).unwrap();
        Ok(T::into_body(body))
    }
}
