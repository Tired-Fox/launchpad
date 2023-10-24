use std::sync::Arc;

use async_trait::async_trait;
use hyper::body::Incoming;

use crate::{body::ParseBody, prelude::Error, server::Parts, Request};

use super::{Body, FromRequestParts, FR, FRP};

#[async_trait]
pub trait FromRequestOrParts<T, S>
where
    Self: Sized + Send,
{
    async fn from_request_or_parts(
        request: hyper::Request<Incoming>,
        parts: Arc<Parts<S>>,
    ) -> Result<Self, Error>;
}

#[async_trait]
impl<S: Send + Sync + 'static, T: FromRequestParts<S>> FromRequestOrParts<FRP, S> for T {
    async fn from_request_or_parts(
        request: hyper::Request<Incoming>,
        parts: Arc<Parts<S>>,
    ) -> Result<Self, Error> {
        T::from_request_parts(&request, parts)
    }
}

#[async_trait]
impl<S: Send + Sync + 'static, T: FromRequest<S>> FromRequestOrParts<FR, S> for T {
    async fn from_request_or_parts(
        request: hyper::Request<Incoming>,
        parts: Arc<Parts<S>>,
    ) -> Result<Self, Error> {
        T::from_request(request, parts).await
    }
}

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
    T: FromRequest<U> + Send,
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
