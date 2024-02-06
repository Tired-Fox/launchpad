//! Credit: https://github.com/de-vri-es/hyper-tungstenite-rs for a good portion of the server side logic.
//!
//! This modules allows [`hyper`](https://docs.rs/hyper) to interact with websocket connections for both the server and the client.
//! A lot of the functionality is backed by [`tungstenite`](https://docs.rs/tungstenite).
//!
//! The [`upgrade`] function allows you to upgrade an HTTP server connection to a websocket connection.
//! The [`connect`] function allows you to connect to the server and obtain a websocket connection.
//! Both of these work together where the client initiates the connection with a handshake and the server responds with
//! the connection.
//!
//! ## [`upgrade`]
//! Will automatically check if the request is able to be upgraded. If not an error response is returned. Otherwise,
//! it returns a hTTP response that *MUST* be sent to the client while the stream can be put into another `tokio::task`
//! to process the websocket messages.
//!
//! ## [`connect`]
//! Will automatically send the request to the server to obtain a websocket connection. It returns a error response
//! if the connection could not be established. Otherwise, it returns the servers HTTP response along with the websocket
//! connection.
//!
//! The goal is to abstract the boilerplate of the headers with `UPGRADE: websocket` and `CONNECTION: upgrade`. On top
//! of this it will abstract `Sec-WebSocket-Key` and `Sec-WebSocket-Version` automatically along with the checks for
//! if the connections are valid.

pub mod prelude {
    pub use futures::sink::SinkExt;
    pub use futures::stream::StreamExt;
}

use anyhow::{Result, bail};
use http_body_util::{Full, Empty};
use hyper::body::{Bytes};
use hyper::{HeaderMap, Request, Response, StatusCode, Uri};
use hyper_util::rt::TokioIo;
use std::task::{Context, Poll};
use std::pin::Pin;
use pin_project_lite::pin_project;

use tungstenite::{Error, error::ProtocolError};
pub use tungstenite::Message;
use tungstenite::handshake::derive_accept_key;
use tungstenite::protocol::{Role, WebSocketConfig};

pub use hyper;
use hyper::header::{CONNECTION, HOST, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_PROTOCOL, SEC_WEBSOCKET_VERSION, UPGRADE};
use hyper::http::{HeaderName, HeaderValue};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
pub use tungstenite;

pub use tokio_tungstenite::WebSocketStream;
use tungstenite::handshake::client::generate_key;

/// A [`WebSocketStream`] that wraps an upgraded HTTP connection from hyper.
pub type WebsocketStream = WebSocketStream<TokioIo<hyper::upgrade::Upgraded>>;

pin_project! {
	/// A future that resolves to a websocket stream when the associated HTTP upgrade completes.
	#[derive(Debug)]
	pub struct Websocket {
		#[pin]
		inner: hyper::upgrade::OnUpgrade,
        role: Role,
		config: Option<WebSocketConfig>,
	}
}

/// Try to upgrade a received `hyper::Request` to a websocket connection.
///
/// The function returns an HTTP response and a future that resolves to the websocket stream.
/// The response body *MUST* be sent to the client before the future can be resolved.
///
/// This functions checks `Sec-WebSocket-Key` and `Sec-WebSocket-Version` headers.
/// It does not inspect the `Origin`, `Sec-WebSocket-Protocol` or `Sec-WebSocket-Extensions` headers.
/// You can inspect the headers manually before calling this function,
/// and modify the response headers appropriately.
///
/// This function also looks at the `Connection` or `Upgrade` headers and makes sure they are set correctly.
///
pub fn upgrade<B>(
    mut request: impl std::borrow::BorrowMut<Request<B>>,
    config: Option<WebSocketConfig>,
) -> Result<(Response<Full<Bytes>>, Websocket)> {
    let request = request.borrow_mut();

    ensure_headers(request.headers(), vec![
        (UPGRADE, ProtocolError::MissingUpgradeWebSocketHeader),
        (CONNECTION, ProtocolError::MissingConnectionUpgradeHeader),
        (SEC_WEBSOCKET_KEY, ProtocolError::MissingSecWebSocketKey),
        (SEC_WEBSOCKET_VERSION, ProtocolError::MissingSecWebSocketVersionHeader),
    ])?;

    let key = request.headers().get("Sec-WebSocket-Key").unwrap();
    if request.headers().get("Sec-WebSocket-Version").map(|v| v.as_bytes()) != Some(b"13") {
        return Err(ProtocolError::MissingSecWebSocketVersionHeader.into());
    }

    let response = Response::builder()
        .status(hyper::StatusCode::SWITCHING_PROTOCOLS)
        .header(hyper::header::CONNECTION, "upgrade")
        .header(hyper::header::UPGRADE, "websocket")
        .header("Sec-WebSocket-Accept", &derive_accept_key(key.as_bytes()))
        .body(Full::<Bytes>::from("switching to websocket protocol"))
        .expect("bug: failed to build response");

    let stream = Websocket {
        inner: hyper::upgrade::on(request),
        role: Role::Server,
        config,
    };

    Ok((response, stream))
}

fn ensure_headers(headers: &HeaderMap<HeaderValue>, keys: Vec<(HeaderName, ProtocolError)>) -> Result<()> {
    for (header, error) in keys {
        if !headers.contains_key(header) {
            return Err(error.into())
        }
    }
    Ok(())
}

/// Try to connect a `hyper::Request` to a websocket connection.
///
/// The function returns an HTTP response and a future that resolves to the websocket stream.
///
/// This functions checks `Sec-WebSocket-Accept` for a valid response key.
/// `Sec-WebSocket-Protocol` is not checked so this should be done manually by the user.
///
/// This function checks  the `Connection` and `Upgrade` headers and ensures that they are valid before returning the
/// connection.
///
pub async fn connect<S: ToString, T: ToString>(
    uri: S,
    protocol: Option<T>,
    config: Option<WebSocketConfig>,
) -> Result<Websocket> {
    let key = generate_key();
    let uri: Uri = uri.to_string().parse()?;
    let authority = uri.authority().unwrap().clone();
    let mut request = Request::get(uri)
        .header(HOST, authority.as_str())
        .header(CONNECTION, "upgrade")
        .header(UPGRADE, "websocket")
        .header(SEC_WEBSOCKET_KEY, key.clone())
        .header(SEC_WEBSOCKET_VERSION, "13");
    if let Some(protocol) = protocol {
        request = request.header(SEC_WEBSOCKET_PROTOCOL, protocol.to_string());
    }

    let request = request.body(Empty::<Bytes>::new())?;

    let host = request.uri().host().unwrap_or("localhost");
    let port: u16 = request.uri().port().map_or(80, |p| p.as_u16());
    let address = format!("{}:{}", host, port);

    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(e) = conn.with_upgrades().await {
            eprintln!("connection failed: {}", e);
        }
    });

    let res = sender.send_request(request).await?;

    // Check response
    ensure_headers(res.headers(), vec![
        (UPGRADE, ProtocolError::MissingUpgradeWebSocketHeader),
        (CONNECTION, ProtocolError::MissingConnectionUpgradeHeader),
        (SEC_WEBSOCKET_ACCEPT, ProtocolError::SecWebSocketAcceptKeyMismatch),
    ])?;

    if res.status() != StatusCode::SWITCHING_PROTOCOLS {
        bail!("server did not switch protocols");
    }

    if res.headers().get(SEC_WEBSOCKET_ACCEPT).unwrap() != &derive_accept_key(key.as_bytes()) {
        return Err(ProtocolError::SecWebSocketAcceptKeyMismatch.into());
    }

    let stream = Websocket {
        inner: hyper::upgrade::on(res),
        role: Role::Client,
        config,
    };

    Ok(stream)
}

/// Check if there is a header of the given name containing the wanted value.
fn header_contains_value(headers: &hyper::HeaderMap, header: impl hyper::header::AsHeaderName, value: impl AsRef<[u8]>) -> bool {
    let value = value.as_ref();
    for header in headers.get_all(header) {
        if header.as_bytes().split(|&c| c == b',').any(|x| trim(x).eq_ignore_ascii_case(value)) {
            return true;
        }
    }
    false
}

fn trim(data: &[u8]) -> &[u8] {
    trim_end(trim_start(data))
}

fn trim_start(data: &[u8]) -> &[u8] {
    if let Some(start) =data.iter().position(|x| !x.is_ascii_whitespace()) {
        &data[start..]
    } else {
        b""
    }
}

fn trim_end(data: &[u8]) -> &[u8] {
    if let Some(last) = data.iter().rposition(|x| !x.is_ascii_whitespace()) {
        &data[..last + 1]
    } else {
        b""
    }
}

impl std::future::Future for Websocket {
    type Output = Result<WebsocketStream, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let role = self.role.clone();
        let this = self.project();
        let upgraded = match this.inner.poll(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(x) => x,
        };

        let upgraded = upgraded.map_err(|_| Error::Protocol(ProtocolError::HandshakeIncomplete))?;

        let stream = WebSocketStream::from_raw_socket(
            TokioIo::new(upgraded),
            role,
            this.config.take(),
        );
        tokio::pin!(stream);

        // The future returned by `from_raw_socket` is always ready.
        // Not sure why it is a future in the first place.
        match stream.as_mut().poll(cx) {
            Poll::Pending => unreachable!("from_raw_socket should always be created ready"),
            Poll::Ready(x) => Poll::Ready(Ok(x)),
        }
    }
}