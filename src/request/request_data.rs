use crate::response::Result;

use super::{body::IntoBody, query::IntoQuery, Body, Query};

pub trait ToParam<T> {
    fn to_param(&mut self) -> T;
}
pub struct RequestData(pub hyper::Uri, pub hyper::Method, pub Vec<u8>);

impl<T: IntoQuery> ToParam<Query<T>> for RequestData {
    fn to_param(&mut self) -> Query<T> {
        match self.0.query() {
            Some(query) => T::into_query(query).unwrap(),
            _ => panic!("No query to parse"),
        }
    }
}

impl<T: IntoQuery> ToParam<Option<Query<T>>> for RequestData {
    fn to_param(&mut self) -> Option<Query<T>> {
        match self.0.query() {
            Some(query) => T::into_query(query).ok(),
            _ => None,
        }
    }
}

impl<T: IntoQuery> ToParam<Result<Query<T>>> for RequestData {
    fn to_param(&mut self) -> Result<Query<T>> {
        match self.0.query() {
            Some(query) => T::into_query(query),
            _ => Err((500, "No query to parse".to_string())),
        }
    }
}

impl<T: IntoBody> ToParam<Body<T>> for RequestData {
    fn to_param(&mut self) -> Body<T> {
        let body = std::str::from_utf8(&self.2[..]).unwrap();
        T::into_body(body).unwrap()
    }
}

impl<T: IntoBody> ToParam<Option<Body<T>>> for RequestData {
    fn to_param(&mut self) -> Option<Body<T>> {
        let body = std::str::from_utf8(&self.2[..]).unwrap();
        T::into_body(body).ok()
    }
}

impl<T: IntoBody> ToParam<Result<Body<T>>> for RequestData {
    fn to_param(&mut self) -> Result<Body<T>> {
        let body = std::str::from_utf8(&self.2[..]).unwrap();
        T::into_body(body)
    }
}
