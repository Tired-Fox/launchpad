use std::{future::Future, pin::Pin, sync::Arc};

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};

use crate::{prelude::IntoResponse, Request};

pub type HandlerFuture = Pin<Box<dyn Future<Output = hyper::Response<Full<Bytes>>> + Send>>;
pub trait Handler: Send + Sync + 'static {
    type Future;

    fn call(&self, request: hyper::Request<Incoming>) -> Self::Future;
    fn referenced(self) -> Arc<dyn Handler<Future = Self::Future> + Send + Sync>;
}

impl<F, Fut, Res> Handler for F
where
    F: Fn(Request) -> Fut + Clone + Sync + Send + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoResponse,
{
    type Future = HandlerFuture;

    fn call(&self, request: hyper::Request<Incoming>) -> Self::Future {
        let refer = self.clone();
        Box::pin(async move { refer(request.into()).await.into_response() })
    }

    fn referenced(self) -> Arc<dyn Handler<Future = Self::Future> + Send + Sync> {
        Arc::new(self)
    }
}
