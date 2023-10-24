use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use hyper::{body::Incoming, Method, StatusCode, Uri, Version};

use crate::{body::ParseBody, prelude::Error, server::Parts, Request};

use super::{Body, FromStateRef, Head, Headers, State};

#[async_trait]
pub trait FromRequest<S>
where
    Self: Sized + Send,
{
    async fn from_request(
        request: hyper::Request<Incoming>,
        parts: Arc<Parts<S>>,
    ) -> Result<Self, Error>;
}

#[async_trait]
impl<T, U> FromRequest<U> for Option<T>
where
    T: FromRequest<U>,
    U: Send + Sync + 'static,
{
    async fn from_request(
        request: hyper::Request<Incoming>,
        parts: Arc<Parts<U>>,
    ) -> Result<Self, Error> {
        Ok(T::from_request(request, parts).await.ok())
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> FromRequest<T> for Body {
    async fn from_request(
        request: hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(Body(request.into_body()))
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> FromRequest<T> for Request {
    async fn from_request(
        request: hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(Request::from(request))
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> FromRequest<T> for String {
    async fn from_request(
        request: hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Request::from(request).text().await
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> FromRequest<T> for Version {
    async fn from_request(
        request: hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(request.version())
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> FromRequest<T> for Head {
    async fn from_request(
        request: hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(Head::new(&request))
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> FromRequest<T> for Method {
    async fn from_request(
        request: hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(request.method().clone())
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> FromRequest<T> for HashMap<String, String> {
    async fn from_request(
        request: hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(request
            .headers()
            .iter()
            .map(|(hn, hv)| (hn.to_string(), hv.to_str().unwrap().to_string()))
            .collect())
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> FromRequest<T> for Headers {
    async fn from_request(
        request: hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(request.headers().clone())
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> FromRequest<T> for Uri {
    async fn from_request(
        request: hyper::Request<Incoming>,
        _parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(request.uri().clone())
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> FromRequest<T> for State<T>
where
    T: FromStateRef<T> + Sync + Clone + Send,
{
    async fn from_request(
        _request: hyper::Request<Incoming>,
        parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        match parts.state() {
            Some(state) => Ok(T::from_state_ref(state)),
            None => Err(Error::from((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failied to parse application state",
            ))),
        }
    }
}
