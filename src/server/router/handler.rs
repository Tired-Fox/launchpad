use std::{future::Future, sync::Arc};

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};

use crate::{
    extract::{FromRequestOrParts, FromRequestParts},
    prelude::IntoResponse,
    server::Parts,
};

use super::route::Captures;
use async_trait::async_trait;

/// Base trait that allows object and methods to be used as a handler.
///
/// This trait is responsible for calling and driving handlers processing a request.
#[async_trait]
pub trait Handler<IN, S: Send + Sync + 'static = ()>: Send + Sync + 'static {
    async fn handle(
        &self,
        request: hyper::Request<Incoming>,
        state: Option<S>,
        catches: Captures,
    ) -> hyper::Response<Full<Bytes>>;
}

#[async_trait]
impl<F, Fut, Res, S> Handler<(), S> for F
where
    F: FnOnce() -> Fut + Clone + Sync + Send + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoResponse,
    S: Send + Sync + 'static,
{
    async fn handle(
        &self,
        _request: hyper::Request<Incoming>,
        _state: Option<S>,
        _captures: Captures,
    ) -> hyper::Response<Full<Bytes>> {
        let handler = self.clone();
        handler().await.into_response()
    }
}

macro_rules! handlers {
    ($([$($types: tt)*]);* $(;)?) => {
        $( handlers!{ $($types)* } )*
    };
    ($($type: ident),* | $last: ident) => {
        paste::paste!{
            #[async_trait]
            impl<F, Fut, Res, S, O, $($type,)* $last> Handler<(O, $($type,)* $last,), S> for F
            where
                F: FnOnce($($type,)* $last) -> Fut + Clone + Sync + Send + 'static,
                Fut: Future<Output = Res> + Send + 'static,
                Res: IntoResponse,
                S: Send + Sync + 'static,
                $(
                    $type: FromRequestParts<S>,
                )*
                $last: FromRequestOrParts<O, S>,
            {
                async fn handle(
                    &self,
                    request: hyper::Request<Incoming>,
                    state: Option<S>,
                    catches: Captures,
                ) -> hyper::Response<Full<Bytes>> {
                    let handler = self.clone();
                    let state = Arc::new(Parts::new(&request, state, catches));

                    let response = {
                        $(
                            let [<$type:lower>] = match $type::from_request_parts(&request, state.clone()) {
                                Ok(value) => value,
                                Err(err) => return err.into_response()
                            };
                        )*

                        let state_clone = state.clone();
                        let [<$last:lower>] = match $last::from_request_or_parts(request, state_clone).await {
                            Ok(value) => value,
                            Err(err) => return err.into_response()
                        };

                        handler(
                            $([<$type:lower>],)*
                            [<$last:lower>],
                        ).await.into_response()
                    };

                    state.cookies().append_response(response)
                }
            }

        }
    }
}

handlers! {
    [|T1];
    [T1|T2];
    [T1,T2|T3];
    [T1,T2,T3|T4];
    [T1,T2,T3,T4|T5];
    [T1,T2,T3,T4,T5|T6];
    [T1,T2,T3,T4,T5,T6|T7];
    [T1,T2,T3,T4,T5,T6,T7|T8];
    [T1,T2,T3,T4,T5,T6,T7,T8|T9];
    [T1,T2,T3,T4,T5,T6,T7,T8,T9|T10];
    [T1,T2,T3,T4,T5,T6,T7,T8,T9,T10|T11];
    [T1,T2,T3,T4,T5,T6,T7,T8,T9,T10,T11|T12];
    [T1,T2,T3,T4,T5,T6,T7,T8,T9,T10,T11,T12|T13];
    [T1,T2,T3,T4,T5,T6,T7,T8,T9,T10,T11,T12,T13|T14];
    [T1,T2,T3,T4,T5,T6,T7,T8,T9,T10,T11,T12,T13,T14|T15];
}
