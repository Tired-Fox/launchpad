pub use form::Form;
pub use html::Html;
pub use json::Json;
pub use query::Query;

pub mod form {
    use std::{
        fmt::{Debug, Display},
        future::Future,
        pin::Pin,
        sync::Arc,
    };

    use hyper::body::Incoming;
    use serde::Deserialize;

    use crate::{
        body::ParseBody, prelude::Error, request::FromRequestBody, server::State, Request,
    };

    pub struct Form<T>(pub T)
    where
        T: Deserialize<'static>;

    impl<T> Debug for Form<T>
    where
        T: Deserialize<'static> + Debug,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Form").field("content", &self.0).finish()
        }
    }

    impl<T> Display for Form<T>
    where
        T: Deserialize<'static> + Display,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl<T> From<T> for Form<T>
    where
        T: Deserialize<'static>,
    {
        fn from(value: T) -> Self {
            Form(value)
        }
    }

    impl<T: Deserialize<'static> + Send> FromRequestBody for Form<T> {
        fn from_request_body(
            request: hyper::Request<Incoming>,
            _: Arc<State>,
        ) -> Pin<Box<dyn Future<Output = Result<Self, Error>> + Send>> {
            Box::pin(async { Request::from(request).form::<T>().await.map(|v| Form(v)) })
        }
    }
}

pub mod query {
    use std::{
        fmt::{Debug, Display},
        sync::Arc,
    };

    use hyper::{body::Incoming, StatusCode};
    use serde::Deserialize;

    use crate::{prelude::Error, request::FromRequest, server::State};

    pub struct Query<T>(pub T)
    where
        T: Deserialize<'static>;

    impl<T> Debug for Query<T>
    where
        T: Deserialize<'static> + Debug,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Query").field("content", &self.0).finish()
        }
    }

    impl<T> Display for Query<T>
    where
        T: Deserialize<'static> + Display,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl<T> From<T> for Query<T>
    where
        T: Deserialize<'static>,
    {
        fn from(value: T) -> Self {
            Query(value)
        }
    }

    impl<T: Deserialize<'static> + Send> FromRequest for Query<T> {
        fn from_request(request: &hyper::Request<Incoming>, _: Arc<State>) -> Result<Self, Error> {
            let query = match request.uri().query() {
                Some(query) => query,
                None => {
                    return Err(Error::from((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Uri does not contain a query",
                    )))
                }
            };

            let static_query = Box::leak(query.to_string().into_boxed_str());
            match serde_qs::from_str::<T>(static_query) {
                Ok(value) => Ok(Query(value)),
                Err(err) => {
                    use serde_qs::Error as qsError;
                    match err {
                        qsError::Unsupported => match serde_plain::from_str::<T>(static_query) {
                            Ok(value) => Ok(Query(value)),
                            _ => Err(Error::from(err)),
                        },
                        err => Err(Error::from(err)),
                    }
                }
            }
        }
    }
}

pub mod html {
    use std::fmt::Debug;
    use std::fmt::Display;

    use crate::body::IntoBody;
    use crate::error::Error;
    use crate::response::IntoResponse;
    use http_body_util::Full;
    use hyper::body::Bytes;

    pub use crate::_html_from as from;
    pub use tela_html::html as new;
    pub use tela_html::prelude::*;
    pub use tela_html::props;

    #[macro_export]
    macro_rules! _html_from {
        ($($html: tt)*) => {
            $crate::Html(
                $crate::html::new! {
                    $($html)*
                }.to_string()
            )
        };
    }

    pub struct Html<T>(pub T)
    where
        T: IntoBody<Full<Bytes>>;

    impl<T> From<T> for Html<T>
    where
        T: IntoBody<Full<Bytes>>,
    {
        fn from(value: T) -> Self {
            Html(value)
        }
    }

    impl<T> Debug for Html<T>
    where
        T: IntoBody<Full<Bytes>> + Debug,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Html").field("content", &self.0).finish()
        }
    }

    impl<T> Display for Html<T>
    where
        T: IntoBody<Full<Bytes>> + Display,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl<T> Default for Html<T>
    where
        T: IntoBody<Full<Bytes>> + Default,
    {
        fn default() -> Self {
            Html(T::default())
        }
    }

    impl<T> IntoResponse for Html<T>
    where
        T: IntoBody<Full<Bytes>>,
    {
        fn into_response(self) -> hyper::Response<Full<Bytes>> {
            match hyper::Response::builder()
                .status(200)
                .header("Content-Type", "text/html")
                .body(self.0.into_body())
            {
                Ok(v) => v,
                Err(e) => Error::from(e).into_response(),
            }
        }
    }

    impl<T> IntoBody<Full<Bytes>> for Html<T>
    where
        T: IntoBody<Full<Bytes>>,
    {
        fn into_body(self) -> Full<Bytes> {
            self.0.into_body()
        }
    }
}

pub mod json {
    use serde::{Deserialize, Serialize};
    pub use serde_json::Value;
    use std::{
        fmt::{Debug, Display},
        future::Future,
        pin::Pin,
        sync::Arc,
    };

    pub use crate::_json_from as from;
    use crate::{
        body::{IntoBody, ParseBody},
        error::Error,
        request::FromRequestBody,
        response::IntoResponse,
        server::State,
        Request,
    };
    use http_body_util::Full;
    use hyper::body::{Bytes, Incoming};
    pub use serde_json::json as new;

    #[macro_export]
    macro_rules! _json_from {
        ($($json: tt)*) => {
            $crate::Json(
                $crate::json::new!($($json)*)
            )
        };
    }

    pub struct Json<T>(pub T)
    where
        T: Serialize + Deserialize<'static>;

    impl<T> Debug for Json<T>
    where
        T: Serialize + Deserialize<'static> + Debug,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Json").field("content", &self.0).finish()
        }
    }

    impl<T> Display for Json<T>
    where
        T: Serialize + Deserialize<'static> + Display,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl<T> IntoBody<Full<Bytes>> for Json<T>
    where
        T: Serialize + Deserialize<'static>,
    {
        fn into_body(self) -> Full<Bytes> {
            match serde_json::to_string(&self.0) {
                Ok(value) => value.into_body(),
                Err(_) => Full::default(),
            }
        }
    }

    impl<T> From<T> for Json<T>
    where
        T: Serialize + Deserialize<'static>,
    {
        fn from(value: T) -> Self {
            Json(value)
        }
    }

    impl<T> IntoResponse for Json<T>
    where
        T: Serialize + Deserialize<'static>,
    {
        fn into_response(self) -> hyper::Response<Full<Bytes>> {
            match hyper::Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(self.into_body())
            {
                Ok(v) => v,
                Err(e) => Error::from(e).into_response(),
            }
        }
    }

    impl<T: Serialize + Deserialize<'static> + Send> FromRequestBody for Json<T> {
        fn from_request_body(
            request: hyper::Request<Incoming>,
            _: Arc<State>,
        ) -> Pin<Box<dyn Future<Output = Result<Self, Error>> + Send>> {
            Box::pin(async { Request::from(request).json::<T>().await.map(|v| Json(v)) })
        }
    }
}
