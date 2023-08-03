use std::{collections::HashMap, convert::Infallible, ffi::OsStr, fs, path::Path, sync::Arc};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Uri};
use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};

use super::errors;
use crate::{
    errors::default_error_page,
    request::{Catch, Endpoint},
    uri::index,
};

/// Commands sent through channel to router
#[derive(Debug)]
pub enum Command {
    Get {
        method: Method,
        path: String,
        response: oneshot::Sender<Option<Route>>,
    },
    Error {
        code: u16,
        response: oneshot::Sender<Option<ErrorHandler>>,
    },
}

#[derive(Debug, Clone)]
pub struct Route(pub Arc<dyn Endpoint>);

#[derive(Debug, Clone)]
pub struct ErrorHandler(pub Arc<dyn Catch>);

#[derive(Clone)]
pub struct Router {
    channel: Option<Sender<Command>>,
    router: HashMap<Method, Vec<Route>>,
    catch: HashMap<u16, ErrorHandler>,
    assets: String,
}
impl Router {
    pub fn new() -> Self {
        Router {
            channel: None,
            router: HashMap::new(),
            catch: HashMap::new(),
            assets: "assets/".to_string(),
        }
    }

    pub fn assets(&mut self, path: String) {
        self.assets = path;
    }

    pub fn catch(&mut self, catch: Arc<dyn Catch>) {
        if !self.catch.contains_key(&catch.code()) {
            self.catch.insert(catch.code(), ErrorHandler(catch));
        }
    }

    pub fn route(&mut self, route: Arc<dyn Endpoint>) {
        for method in route.methods() {
            if !self.router.contains_key(&method) {
                self.router.insert(method.clone(), Vec::new());
            }
            self.router
                .get_mut(&method)
                .unwrap()
                .push(Route(route.clone()));
        }
    }

    /// Start listener thread for handling access to router
    ///
    /// Creates mpsc channel and returns Sender handle. The thread that this method
    /// creates is the only instance of the router that should exists.
    pub fn serve_routes(&mut self) {
        let (tx, mut rx) = mpsc::channel::<Command>(32);
        let router = self.router.clone();
        let catch = self.catch.clone();

        tokio::spawn(async move {
            'watcher: while let Some(cmd) = rx.recv().await {
                use Command::*;

                match cmd {
                    Get {
                        method,
                        path,
                        response,
                    } => {
                        match router.get(&method) {
                            Some(data) => {
                                match index(
                                    &path,
                                    &data.iter().map(|r| r.0.path()).collect::<Vec<String>>(),
                                ) {
                                    Some(index) => {
                                        response.send(Some(data[index].clone())).unwrap();
                                        continue 'watcher;
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        };
                        response.send(None).unwrap();
                    }
                    Error { code, response } => {
                        if catch.contains_key(&code) {
                            response
                                .send(catch.get(&code).map(|eh| eh.clone()))
                                .unwrap()
                        } else if catch.contains_key(&0) {
                            response.send(catch.get(&0).map(|eh| eh.clone())).unwrap()
                        } else {
                            response.send(None).unwrap()
                        }
                    }
                }
            }
        });

        self.channel = Some(tx);
    }

    async fn error(
        &self,
        uri: &Uri,
        method: &Method,
        body: &Vec<u8>,
        code: u16,
        reason: String,
        channel: Sender<Command>,
    ) -> std::result::Result<hyper::Response<Full<Bytes>>, Infallible> {
        let (error_tx, error_rx) = oneshot::channel();
        match channel
            .send(Command::Error {
                code: code.clone(),
                response: error_tx,
            })
            .await
        {
            Ok(_) => {}
            Err(error) => eprintln!("{:?}", error),
        };

        match error_rx.await.unwrap() {
            Some(ErrorHandler(handler)) => {
                match handler.execute(
                    code.clone(),
                    errors::MESSAGES.get(&code).unwrap_or(&"").to_string(),
                    reason.clone(),
                ) {
                    Ok(response) => {
                        Router::log_request(
                            &uri.path().to_string(),
                            &method.clone(),
                            &response.status().into(),
                        );
                        Ok(response)
                    }
                    Err((code, reason)) => {
                        Router::log_request(&uri.path().to_string(), method, &code);
                        Ok(default_error_page(
                            &code,
                            &reason,
                            method,
                            uri,
                            std::str::from_utf8(body).unwrap_or("").to_string(),
                        ))
                    }
                }
            }
            None => {
                Router::log_request(&uri.path().to_string(), method, &code);
                Ok(default_error_page(
                    &code,
                    &reason,
                    method,
                    uri,
                    std::str::from_utf8(body).unwrap_or("").to_string(),
                ))
            }
        }
    }

    fn log_request(path: &String, method: &Method, status: &u16) {
        #[cfg(debug_assertions)]
        eprintln!(
            "  {}(\x1b[3{}m{}\x1b[39m) \x1b[32m{:?}\x1b[0m",
            method,
            match status {
                100..=199 => 6,
                200..=299 => 2,
                300..=399 => 5,
                400..=499 => 1,
                500..=599 => 3,
                _ => 7,
            },
            status,
            path
        )
    }

    pub async fn parse(
        &self,
        request: hyper::Request<hyper::body::Incoming>,
    ) -> Result<hyper::Response<Full<Bytes>>, Infallible> {
        // Get all needed information from request
        let mut uri = request.uri().clone();
        let method = request.method().clone();
        // Can be used for validation, authentication, and other features
        let _headers = request.headers().clone();
        let mut body = request.collect().await.unwrap().to_bytes().to_vec();

        let (endpoint_tx, endpoint_rx) = oneshot::channel();
        match &self.channel {
            Some(channel) => {
                let path = format!("{}{}", self.assets, uri.path());
                let path = Path::new(&path);
                if let Some(extension) = path.extension().and_then(OsStr::to_str) {
                    match fs::read_to_string(path) {
                        Ok(text) => {
                            Router::log_request(&uri.path().to_string(), &method, &200);
                            let mut builder = hyper::Response::builder().status(200);

                            match mime_guess::from_ext(extension).first() {
                                Some(mime) => {
                                    builder = builder.header("Content-Type", mime.to_string())
                                }
                                _ => {}
                            };

                            return Ok(builder.body(Full::new(Bytes::from(text))).unwrap());
                        }
                        _ => {
                            Router::log_request(&uri.path().to_string(), &method, &404);
                            return Ok(default_error_page(
                                &404,
                                &"File not found".to_string(),
                                &method,
                                &uri,
                                std::str::from_utf8(body.as_slice())
                                    .unwrap_or("")
                                    .to_string(),
                            ));
                        }
                    }
                }

                match channel
                    .send(Command::Get {
                        method: method.clone(),
                        path: uri.path().to_string(),
                        response: endpoint_tx,
                    })
                    .await
                {
                    Ok(_) => {}
                    Err(error) => eprintln!("{}", error),
                };

                match endpoint_rx.await.unwrap() {
                    Some(Route(endpoint)) => match endpoint.execute(&method, &mut uri, &mut body) {
                        Ok(response) => {
                            Router::log_request(
                                &uri.path().to_string(),
                                &method,
                                &response.status().into(),
                            );
                            Ok(response)
                        }
                        Err((code, reason)) => {
                            self.error(&uri, &method, &body, code, reason, channel.clone())
                                .await
                        }
                    },
                    None => {
                        self.error(
                            &uri,
                            &method,
                            &body,
                            404,
                            "Page not found in router".to_string(),
                            channel.clone(),
                        )
                        .await
                    }
                }
            }
            _ => panic!("Unable to communicate with router"),
        }
    }
}
