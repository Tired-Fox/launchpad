use crate::response::Result;
use serde::Deserialize;

pub trait IntoBody {
    fn into_body(body: &str) -> Result<Body<Self>>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy)]
pub struct Body<T: IntoBody>(pub T);
impl<T: IntoBody> Body<T> {
    pub fn extract(body: Vec<u8>) -> Result<Self>
    where
        Self: Sized,
    {
        let body = std::str::from_utf8(&body[..]).unwrap();
        T::into_body(body)
    }
}

impl<'a, T: Deserialize<'a>> IntoBody for T {
    fn into_body(body: &str) -> Result<Body<Self>>
    where
        Self: Sized,
    {
        let body = body.to_string();
        match serde_json::from_str::<T>(Box::leak(body.clone().into_boxed_str())) {
            Ok(result) => Ok(Body(result)),
            Err(_) => match serde_plain::from_str::<T>(Box::leak(body.into_boxed_str())) {
                Ok(result) => Ok(Body(result)),
                Err(_) => Err((500, "Failed to parse body from request".to_string())),
            },
        }
    }
}
