use std::{path::PathBuf, convert::Infallible, fs};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Uri, HeaderMap, http::HeaderValue};
use tokio::sync::{oneshot, mpsc::Sender};

use crate::{Response, ROOT};

use super::server::Command;

/// Ensure there is a meta charset=utf-8 tag in the html response. If not inject it.
fn format_body(body: String) -> String {
    let re = regex::Regex::new("(?i)<meta +charset=\"UTF-8\" *>").unwrap();
    let response = match re.find(body.as_str()) {
        None => format!("<head><meta charset=\"UTF-8\"></head>\n{}", body),
        Some(_) => body,
    };
    response
}

/// Build an error response and ensure that the redirect uri is set for 301-308 codes
fn build_error(code: u16, msg: String, reason: String, fb: bool) -> hyper::Response<Full<Bytes>> {
    let mut response = hyper::Response::builder().status(code);
    match code {
        301 | 308 | 302 | 303 | 307 => {
            if reason == String::new() {
                panic!("Redirect responses require an error reason that is the Location")
            }
            response = response
                .header("LaunchPad-Reason", "Redirect")
                .header("Location", reason);
        }
        _ => {
            response = response.header("LaunchPad-Reason", reason);
        }
    }

    response
        .body(http_body_util::Full::new(bytes::Bytes::from(
            match msg.len() == 0 {
                false => match fb {
                    true => format_body(msg),
                    _ => msg,
                },
                _ => msg,
            },
        )))
        .unwrap()
}

/// Helper for constructing success responses
macro_rules! response {
    ($data: expr) => {
        hyper::Response::new(http_body_util::Full::new(bytes::Bytes::from(format_body(
            $data,
        ))))
    };
    (FILE $data: expr) => {
        hyper::Response::new(http_body_util::Full::new(bytes::Bytes::from($data)))
    };
}

/// Helper for constructing error responses
macro_rules! error {
    ($code: expr, $reason: expr, $msg: expr) => {
        build_error($code, $msg, $reason, true)
    };
    ($code: expr, $reason: expr) => {
        build_error($code, String::new(), $reason, true)
    };
    ($code: expr) => {
        build_error($code, String::new(), String::new(), true)
    };
    (RAW $code: expr, $reason: expr, $msg: expr) => {
        build_error($code, $msg, $reason, false)
    };
    (RAW $code: expr, $reason: expr) => {
        build_error($code, String::new(), $reason, false)
    };
    (RAW $code: expr) => {
        build_error($code, String::new(), String::new(), false)
    };
}

/// Route handler that will parse a request and serve either a file or endpoint
///
/// If the file or endpoint is not found then it will respond with a 404 not found
pub(crate) struct RouteHandler(Sender<Command>);
impl RouteHandler {
    pub fn new(router: Sender<Command>) -> Self {
        RouteHandler(router)
    }

    /// Formats and builds the debug details element on the error page
    fn format_request_debug(
        message: String,
        method: &hyper::Method,
        uri: &hyper::Uri,
        body: &Bytes,
    ) -> String {
        format!(
            r#"<pre>
    {}
        <em>Request: <strong>{} → '{}</strong>'</em>
        <em>Body: <strong>{}</strong></em>
    </pre>"#,
            message,
            method,
            uri,
            String::from_utf8(body.to_vec()).unwrap()
        )
    }

    fn router(&self) -> &Sender<Command> {
        &self.0
    }

    /// Handle and serve a file request
    async fn handle_file(
        &self,
        path: PathBuf,
        uri: Uri,
        method: Method,
        body: Bytes
    ) -> hyper::Response<Full<Bytes>>  {

        if !path.is_file() {
            if path.to_str().unwrap().ends_with("html") {
                let (resp_tx, resp_rx) = oneshot::channel();
                self.router()
                    .send(Command::Error {
                        code: 404,
                        reason: RouteHandler::format_request_debug(
                            format!("Could not find file {:?}", path.to_string_lossy()),
                            &method,
                            &uri,
                            &body,
                        ),
                        response: resp_tx,
                    })
                    .await
                    .unwrap();
                let message = resp_rx.await.unwrap();
                error!(
                    404,
                    format!("File not found: {}", path.to_string_lossy()),
                    message
                )
            } else {
                error!(RAW 404, "Path not in router".to_string())
            }
        } else {
            response!(
                FILE
                fs::read(path).expect("Could not read from file")
            )
        }
    }

    /// Handle and serve a endpoint request
    async fn handle_endpoint(
        &self,
        path: String,
        path_buff: PathBuf,
        uri: Uri,
        headers: HeaderMap<HeaderValue>,
        method: Method,
        body: Bytes
    ) -> hyper::Response<Full<Bytes>> {

        // Get route/endpoint from router
        let (resp_tx, resp_rx) = oneshot::channel();
        self.router()
            .send(Command::Get {
                method: method.clone(),
                path: path.clone(),
                response: resp_tx,
            })
            .await
            .unwrap();
        let endpoint = resp_rx.await.unwrap();

        match endpoint {
            // Endpoint exists so execute it and process the response
            Some(endpoint) => match endpoint.endpoint().execute(&uri, &headers, &body) {
                Response::Success(content_type, data) => hyper::Response::builder()
                    .status(200)
                    .header("Content-Type", content_type)
                    .body(Full::new(data))
                    .unwrap(),
                Response::Error(code, message) => {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    self.router()
                        .send(Command::Error {
                            code: code.clone(),
                            reason: RouteHandler::format_request_debug(
                                match &message {
                                    Some(msg) => msg.clone(),
                                    _ => String::from("A user defined error occured"),
                                },
                                &method,
                                &uri,
                                &body,
                            ),
                            response: resp_tx,
                        })
                        .await
                        .unwrap();

                    let body = resp_rx.await.unwrap();
                    match message {
                        Some(msg) => error!(code, msg, body),
                        _ => error!(code, "".to_string(), body),
                    }
                }
            },
            // Endpoint doesn't exists so respond with 404 not found
            _ => {
                let (resp_tx, resp_rx) = oneshot::channel();
                println!("{}", path_buff.display());
                self.router()
                    .send(Command::Error {
                        code: 404,
                        reason: RouteHandler::format_request_debug(
                            format!(
                                "<span class=\"path\"><strong>{}{}</strong></span> not found in router",
                                match path_buff.display().to_string().as_str() {
                                    "" => "/",
                                    _ => ""
                                },
                                path_buff.display(),
                            ),
                            &method,
                            &uri,
                            &body
                        ),
                        response: resp_tx,
                    })
                    .await
                    .unwrap();
                let message = resp_rx.await.unwrap();
                error!(404, "path not found in router".to_string(), message)
            }
        }
    }

    pub async fn parse(
        &self,
        req: hyper::Request<hyper::body::Incoming>,
    ) -> Result<hyper::Response<Full<Bytes>>, Infallible> {

        // Contruct path to match against
        let mut path = req.uri().path().to_string();
        if path.ends_with("/") {
            path.pop();
        }
        let path_buff = PathBuf::from(path.clone());

        // Extract data from request
        let uri: Uri = req.uri().clone();
        let headers: HeaderMap<HeaderValue> = req.headers().clone();
        let method: Method = req.method().clone();
        let body: Bytes = req.collect().await.unwrap().to_bytes();

        // Serve endpoint or a file
        let response = match path_buff.extension() {
            None => {
                self.handle_endpoint(
                    path.clone(),
                    path_buff,
                    uri,
                    headers,
                    method.clone(),
                    body
                ).await
            }
            Some(_) => {
                self.handle_file(
                    PathBuf::from(format!("{}{}", ROOT, path)),
                    uri,
                    method.clone(),
                    body
                ).await
            }
        };

        // Log the request result in debug mode
        #[cfg(debug_assertions)]
        {
            let code = response.status().as_u16();
            println!(
                "  {} ({}) {}",
                match method {
                    Method::GET => format!("\x1b[36mGET\x1b[39m"),
                    Method::POST => format!("\x1b[35mPOST\x1b[39m"),
                    Method::DELETE => format!("\x1b[31mDELETE\x1b[39m"),
                    val => format!("{}", val),
                },
                match code {
                    100..=199 => format!("\x1b[36m{}\x1b[39m", code),
                    200..=299 => format!("\x1b[32m{}\x1b[39m", code),
                    300..=399 => format!("\x1b[33m{}\x1b[39m", code),
                    400..=499 => format!("\x1b[31m{}\x1b[39m", code),
                    500..=599 => format!("\x1b[35m{}\x1b[39m", code),
                    _ => code.to_string(),
                },
                format!("\x1b[32m'{}'\x1b[39m", path)
            );
        }

        Ok(response)
    }
}
