use std::{future::Future, pin::Pin, sync::Arc};

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};

use crate::{
    prelude::IntoResponse,
    request::{FromRequest, FromRequestBody},
    server::State,
};

pub type HandlerFuture = Pin<Box<dyn Future<Output = hyper::Response<Full<Bytes>>> + Send>>;
pub trait Handler<IN = ()>: Send + Sync + 'static {
    fn handle_request(
        &self,
        request: hyper::Request<Incoming>,
    ) -> Pin<Box<dyn Future<Output = hyper::Response<Full<Bytes>>> + Send + 'static>>;
}

impl<F, Fut, Res> Handler<((),)> for F
where
    F: FnOnce() -> Fut + Clone + Sync + Send + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    Res: IntoResponse,
{
    fn handle_request(
        &self,
        _: hyper::Request<Incoming>,
    ) -> Pin<Box<dyn Future<Output = hyper::Response<Full<Bytes>>> + Send + 'static>> {
        let refer = self.clone();
        Box::pin(async move { refer().await.into_response() })
    }
}

macro_rules! handlers {
    ($([$($types: tt)*]);* $(;)?) => {
        $( handlers!{ $($types)* } )*
    };
    ($($type: ident),* | $last: ident) => {
        paste::paste!{
            impl<F, Fut, Res, $($type,)* $last> Handler<($($type,)* $last,)> for F
            where
                F: FnOnce($($type,)* $last) -> Fut + Clone + Sync + Send + 'static,
                Fut: Future<Output = Res> + Send + 'static,
                Res: IntoResponse,
                $(
                    $type: FromRequest,
                )*
                $last: FromRequestBody,
            {
                fn handle_request(
                    &self,
                    request: hyper::Request<Incoming>,
                ) -> Pin<Box<dyn Future<Output = hyper::Response<Full<Bytes>>> + Send + 'static>> {
                    let refer = self.clone();
                    Box::pin(async move {
                        let state = Arc::new(State::new(&request));

                        let response = {
                            $(
                                let [<$type:lower>] = match $type::from_request(&request, state.clone()) {
                                    Ok(value) => value,
                                    Err(err) => return err.into_response()
                                };
                            )*

                            let state_clone = state.clone();
                            let [<$last:lower>] = match $last::from_request_body(request, state_clone).await {
                                Ok(value) => value,
                                Err(err) => return err.into_response()
                            };

                            refer(
                                $([<$type:lower>],)*
                                [<$last:lower>],
                            ).await.into_response()
                        };

                        state.cookies().append_response(response)
                    })
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
}
