use std::{
    collections::HashMap,
    fmt::Display,
    str::FromStr,
    sync::{Arc, RwLock},
};

use chrono::{naive::NaiveDateTime, FixedOffset, TimeZone};
pub use chrono::{DateTime, Duration, Local};
use chrono_tz::GMT;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};

use crate::{extract::FromRequestParts, prelude::Error, server::Parts};

#[derive(Default, Clone, Debug)]
pub enum SameSite {
    Strict,
    Lax,
    #[default]
    None,
}

impl Display for SameSite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Lax => "Lax",
                Self::Strict => "Strict",
                Self::None => "",
            }
        )
    }
}

pub trait IntoCookieExpiration {
    fn into_cookie_expiration(self) -> DateTime<FixedOffset>;
}

impl IntoCookieExpiration for &str {
    fn into_cookie_expiration(self) -> DateTime<FixedOffset> {
        let naive = NaiveDateTime::from_str(self).unwrap();
        let local = Local.from_local_datetime(&naive).unwrap();
        local.with_timezone(&GMT).fixed_offset()
    }
}

impl IntoCookieExpiration for String {
    fn into_cookie_expiration(self) -> DateTime<FixedOffset> {
        let naive = NaiveDateTime::from_str(self.as_str()).unwrap();
        let local = Local.from_local_datetime(&naive).unwrap();
        local.with_timezone(&GMT).fixed_offset()
    }
}

impl IntoCookieExpiration for DateTime<Local> {
    fn into_cookie_expiration(self) -> DateTime<FixedOffset> {
        self.with_timezone(&GMT).fixed_offset()
    }
}

#[derive(Default)]
pub struct Builder(Cookie);
impl Builder {
    pub fn new(content: String) -> Self {
        Builder(Cookie {
            content,
            ..Default::default()
        })
    }

    pub fn domain(mut self, domain: &str) -> Self {
        self.0.domain = Some(domain.to_string());
        self
    }
    pub fn expires<T: IntoCookieExpiration>(mut self, expires: T) -> Self {
        self.0.expires = Some(expires.into_cookie_expiration());
        self
    }
    pub fn max_age(mut self, max_age: i32) -> Self {
        self.0.max_age = Some(max_age);
        self
    }
    pub fn path(mut self, path: &str) -> Self {
        self.0.path = Some(path.to_string());
        self
    }
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        if let SameSite::None = same_site {
            self.0.secure = true;
        }
        self.0.same_site = Some(same_site);
        self
    }
    pub fn http_only(mut self) -> Self {
        self.0.http_only = true;
        self
    }
    pub fn partitioned(mut self) -> Self {
        self.0.partitioned = true;
        self
    }
    pub fn secure(mut self) -> Self {
        self.0.secure = true;
        self
    }

    pub fn finish(self) -> Cookie {
        self.0
    }
}

/// A data representation of a Set-Cookie header.
///
/// This object allows the value and properties of a cookie to be built up.
/// This doesn't include the name of the cookies. However, if `cookie.stringify("Name")` is called
/// on the cookie it will take the cookie name and generate the header string value. Stringify
/// the cookie will also replace all `;` in the cookie value with `%3B` to help reduce the
/// limitations of the cookie value.
///
/// # Example
/// ```
/// // Don't forget to enable the `cookies` feature flag
///
/// // The `Local` and `Durations` objects are from the `chrono` crate
/// use tela::cookie::{Cookie, Local, Duration, SameSite};
///
/// Cookie::new(3);
///
/// // or
///
/// Cookie::builder("value")
///    .domain("example.com")
///    .expires(Local::now() + Duration::hours(12))
///    .expires("03/22/2023 10:44:12") // Assumed to be local time zone
///    .http_only()
///    .max_age(-1)
///    .partitioned()
///    .path("/sub/path/here")
///    .same_site(SameSite::Strict)
///    .secure()
///    .finish();
///
/// ```
///
/// Refer to [Set-Cookie](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie) for
/// more on what each value means
#[derive(Default, Clone, Debug)]
pub struct Cookie {
    content: String,
    domain: Option<String>,
    expires: Option<DateTime<FixedOffset>>,
    max_age: Option<i32>,
    path: Option<String>,
    // This forces secure to be toggled on
    same_site: Option<SameSite>,

    http_only: bool,
    partitioned: bool,
    secure: bool,
}

impl Cookie {
    pub fn new<T: ToString>(value: T) -> Self {
        Cookie {
            content: value.to_string(),
            ..Default::default()
        }
    }

    pub fn delete() -> Self {
        Cookie {
            max_age: Some(-1),
            ..Default::default()
        }
    }

    pub fn builder<T: ToString>(content: T) -> Builder {
        Builder::new(content.to_string())
    }

    pub fn stringify(&self, name: &str) -> String {
        let mut secure = self.secure;
        if let Some(SameSite::None) = self.same_site {
            secure = true;
        }
        format!(
            "{}={}{}{}{}{}{}{}{}{}",
            name,
            self.content.replace(";", "%3B"),
            self.domain
                .as_ref()
                .map(|v| format!(";Domain={}", v))
                .unwrap_or(String::new()),
            self.expires
                .as_ref()
                .map(|v| format!(";Expires={}", v.format("%a, %d %b %Y %H:%M:%S GMT")))
                .unwrap_or(String::new()),
            self.max_age
                .as_ref()
                .map(|v| format!(";Max-Age={}", v))
                .unwrap_or(String::new()),
            self.path
                .as_ref()
                .map(|v| format!(";Path={}", v))
                .unwrap_or(String::new()),
            self.same_site
                .as_ref()
                .map(|v| format!(";SameSite={}", v))
                .unwrap_or(String::new()),
            match secure {
                true => ";Secure",
                false => "",
            },
            match self.partitioned {
                true => ";Partitioned",
                false => "",
            },
            match self.http_only {
                true => ";HttpOnly",
                false => "",
            },
        )
    }
}

/// Utility object to read request cookies and send new cookies in a response.
///
/// The cookies being sent back in the response creates a lock whenever a cookie is added and is release when
/// the cookie is done being added. Each CookieJar is unique to each request so this shouldn't be an issue as
/// long as this is kept in mind while creating the endpoint.
///
/// The escape code `%3B` is converted to `;` along with `;` to `%3B` to allow the limitations of
/// what can be stored in the cookies
///
/// Supported actions include: `get`, `set`, and `delete`.
/// - `Get` will return the specific cookie by name if it exists.
/// - `Set` will take a cookie name along with a `Cookie` and store it in a map. This map generates
///     `Set-Cookie` headers for each new cookie being sent in the response.
/// - `Delete` is the same as `set` except it will create the cookie for you and set the `Max-Age`
///     property to -1 to tell the browser to immediately delete the cookie.
///
/// # Example
/// ```
/// // Don't forget to toggle the `cookies` feature flag
///
/// use tela::cookie::{Cookie, CookieJar  };
/// use tela::server::{Router, router::get, Server, Socket};
///
///
/// async fn handler(mut cookies: CookieJar) {
///     match cookies.get("TelaExample") {
///         Some(cookie) => cookies.delete("TelaExample"),
///         None => cookies.set("TelaExample", Cookie::new(1))
///     };
/// }
///
/// async fn main() {
///     Server::new().serve(
///         Socket::Local(3000),
///         Router::new()
///             .route("/", get(handler))
///     ).await;
/// }
/// ```
#[derive(Default, Clone, Debug)]
pub struct CookieJar {
    request: Arc<HashMap<String, String>>,
    response: Arc<RwLock<HashMap<String, Cookie>>>,
}

impl CookieJar {
    pub fn new(cookies: String) -> Self {
        CookieJar {
            request: Arc::new(
                cookies
                    .split(";")
                    .filter_map(|v| {
                        if v.is_empty() {
                            return None;
                        }
                        let v = v.split_once("=").unwrap();
                        Some((v.0.trim().to_string(), v.1.replace("%3B", ";")))
                    })
                    .collect(),
            ),
            ..Default::default()
        }
    }

    pub fn get(&self, name: &str) -> Option<String> {
        match self.request.get(name) {
            Some(v) => Some(v.clone()),
            None => match self.response.read().unwrap().get(name) {
                Some(v) => Some(v.content.clone()),
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: &str, value: Cookie) {
        self.response
            .write()
            .unwrap()
            .insert(name.to_string(), value);
    }

    pub fn delete(&mut self, name: &str) {
        self.response
            .write()
            .unwrap()
            .insert(name.to_string(), Cookie::delete());
    }

    pub fn append_response(
        &self,
        mut response: hyper::Response<Full<Bytes>>,
    ) -> hyper::Response<Full<Bytes>> {
        if !self.response.read().unwrap().is_empty() {
            let headers = response.headers_mut();
            for (key, cookie) in self.response.read().unwrap().iter() {
                headers.append(
                    hyper::header::SET_COOKIE,
                    cookie.stringify(key).parse().unwrap(),
                );
            }
        }
        response
    }
}

impl<T: Send + Sync + 'static> FromRequestParts<T> for CookieJar {
    fn from_request_parts(
        _request: &hyper::Request<Incoming>,
        parts: Arc<Parts<T>>,
    ) -> Result<Self, Error> {
        Ok(parts.cookies().clone())
    }
}
