use std::{collections::HashMap, sync::Arc};

use hyper::{body::Incoming, Method, StatusCode, Uri, Version};

use crate::{prelude::Error, server::Parts};

use super::{Head, Headers, State, ToState};

pub trait FromRequestParts<T>
where
    Self: Send + Sized,
    T: Send + Sync + Clone + 'static,
{
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        parts: Arc<Parts<T>>,
    ) -> Result<Self, Error>;
}

impl<T, U: Send + Sync + Clone + 'static> FromRequestParts<U> for Option<T>
where
    T: FromRequestParts<U>,
{
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        parts: Arc<Parts<U>>,
    ) -> Result<Self, Error> {
        Ok(T::from_request_parts(request, parts).ok())
    }
}

impl<T: Send + Sync + Clone + 'static> FromRequestParts<T> for Version {
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(request.version())
    }
}

impl<T: Send + Sync + Clone + 'static> FromRequestParts<T> for Head {
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(Head::new(request))
    }
}

impl<T: Send + Sync + Clone + 'static> FromRequestParts<T> for Method {
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(request.method().clone())
    }
}

impl<T: Send + Sync + Clone + 'static> FromRequestParts<T> for HashMap<String, String> {
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(request
            .headers()
            .iter()
            .map(|(hn, hv)| (hn.to_string(), hv.to_str().unwrap().to_string()))
            .collect())
    }
}

impl<T: Send + Sync + Clone + 'static> FromRequestParts<T> for Headers {
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(request.headers().clone())
    }
}

impl<T: Send + Sync + Clone + 'static> FromRequestParts<T> for Uri {
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(request.uri().clone())
    }
}

impl<T: Send + Sync + Clone + 'static> FromRequestParts<T> for State<T> {
    fn from_request_parts(
        _request: &hyper::Request<Incoming>,
        parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        match parts.state() {
            Some(state) => Ok(state.to_state()),
            None => Err(Error::from((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse application state",
            ))),
        }
    }
}
