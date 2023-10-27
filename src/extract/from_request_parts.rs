use std::{collections::HashMap, sync::Arc};

use hyper::{body::Incoming, Method, StatusCode, Uri, Version};

use crate::server::{FromStateRef, State};
use crate::{prelude::Error, server::Parts};

use super::{Head, Headers};

pub trait FromRequestParts<T = ()>
where
    Self: Send + Sized,
{
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        parts: Arc<Parts<T>>,
    ) -> Result<Self, Error>;
}

impl<T, U> FromRequestParts<U> for Option<T>
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

impl<T> FromRequestParts<T> for Version {
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(request.version())
    }
}

impl<T> FromRequestParts<T> for Head {
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(Head::new(request))
    }
}

impl<T> FromRequestParts<T> for Method {
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(request.method().clone())
    }
}

impl<T> FromRequestParts<T> for HashMap<String, String> {
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

impl<T> FromRequestParts<T> for Headers {
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(request.headers().clone())
    }
}

impl<T> FromRequestParts<T> for Uri {
    fn from_request_parts(
        request: &hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(request.uri().clone())
    }
}

impl<S, T> FromRequestParts<S> for State<T>
where
    T: FromStateRef<S> + Send + Clone + 'static,
    S: Clone,
{
    fn from_request_parts(
        _request: &hyper::Request<Incoming>,
        parts: Arc<Parts<S>>,
    ) -> Result<Self, Error> {
        match parts.state() {
            Some(state) => Ok(T::from_state_ref(state)),
            None => Err(Error::from((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse application state: No state found",
            ))),
        }
    }
}
