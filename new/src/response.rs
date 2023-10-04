use http_body_util::Full;
use hyper::{body::Bytes, Response};

pub trait IntoResponse {
    fn into_response(self) -> Response<Full<Bytes>>;
}

impl IntoResponse for Response<Full<Bytes>> {
    fn into_response(self) -> Response<Full<Bytes>> {
        self
    }
}

impl IntoResponse for &str {
    fn into_response(self) -> Response<Full<Bytes>> {
        Response::builder()
            .status(200)
            .body(Full::new(Bytes::from(self.to_string())))
            .unwrap()
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response<Full<Bytes>> {
        Response::builder()
            .status(200)
            .body(Full::new(Bytes::from(self)))
            .unwrap()
    }
}
