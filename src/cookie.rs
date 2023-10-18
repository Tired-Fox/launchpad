use std::{
    collections::{hash_map, HashMap},
    fmt::Display,
    str::FromStr,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use chrono::{naive::NaiveDateTime, DateTime, FixedOffset, Local, TimeZone};
use chrono_tz::GMT;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};

use crate::{prelude::Error, request::FromRequest, server::State};

#[derive(Default, Clone)]
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

impl IntoCookieExpiration for DateTime<Local> {
    fn into_cookie_expiration(self) -> DateTime<FixedOffset> {
        self.with_timezone(&GMT).fixed_offset()
    }
}

#[derive(Default)]
pub struct Builder(Cookie);
impl Builder {
    pub fn domain(mut self, domain: String) -> Self {
        self.0.domain = Some(domain);
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

    pub fn eat<T: ToString>(mut self, value: T) -> Cookie {
        self.0.content = value.to_string();
        self.0
    }
}

#[derive(Default, Clone)]
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

    pub fn builder() -> Builder {
        Builder::default()
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

#[derive(Default, Clone)]
pub struct Cookies {
    request: Arc<HashMap<String, String>>,
    response: Arc<RwLock<HashMap<String, Cookie>>>,
}

impl Cookies {
    pub fn new(cookies: String) -> Self {
        Cookies {
            request: Arc::new(
                cookies
                    .split(";")
                    .filter_map(|v| {
                        if v.is_empty() {
                            return None;
                        }
                        let v = v.split_once("=").unwrap();
                        Some((v.0.to_string(), v.1.replace("%3B", ";")))
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

impl FromRequest for Cookies {
    fn from_request(_request: &hyper::Request<Incoming>, state: Arc<State>) -> Result<Self, Error> {
        Ok(state.cookies().clone())
    }
}
