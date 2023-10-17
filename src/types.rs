pub use form::Form;
pub use html::Html;
pub use json::Json;

pub mod form {
    use serde::Deserialize;

    pub struct Form<T>(pub T)
    where
        T: Deserialize<'static>;

    impl<T> From<T> for Form<T>
    where
        T: Deserialize<'static>,
    {
        fn from(value: T) -> Self {
            Form(value)
        }
    }
}

pub mod html {
    use std::fmt::Display;

    pub use crate::_html_from as from;
    use crate::body::IntoBody;
    use crate::error::Error;
    use crate::response::IntoResponse;
    use http_body_util::Full;
    use hyper::body::Bytes;
    pub use tela_html::html as new;
    pub use tela_html::prelude::*;

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
    use std::fmt::Display;

    pub use crate::_json_from as from;
    use crate::{body::IntoBody, error::Error, response::IntoResponse};
    use http_body_util::Full;
    use hyper::body::Bytes;
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
}
