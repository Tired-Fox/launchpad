use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

use hyper::body::Incoming;

use crate::{prelude::IntoResponse, response::Body, Request};

pub type HandlerFuture = Pin<Box<dyn Future<Output = hyper::Response<Body>> + Send>>;
pub trait Handler: Send + Sync + 'static {
    type Future;

    fn call(&self, request: hyper::Request<Incoming>) -> Self::Future;
    fn arced(self) -> Arc<dyn Handler<Future = Self::Future> + Send + Sync>;
}

impl<F, Fut, Res> Handler for F
where
    F: Fn(Request) -> Fut + Clone + Sync + Send + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoResponse,
{
    type Future = HandlerFuture;

    fn call(&self, request: hyper::Request<Incoming>) -> Self::Future {
        let _ = PhantomData::<F>;
        let refer = self.clone();
        Box::pin(async move { refer(request.into()).await.into_response() })
    }

    fn arced(self) -> Arc<dyn Handler<Future = Self::Future> + Send + Sync> {
        Arc::new(self)
    }
}
