use std::{error::Error, net::SocketAddr, sync::Arc};

use hyper::{server::conn::http1, service::service_fn};
use tokio::net::TcpListener;

use crate::{
    prelude::{Catch, Endpoint},
    support::TokioIo,
    Router,
};

pub trait IntoSocketAddr {
    fn into_socket_addr(self) -> SocketAddr;
}

impl IntoSocketAddr for u16 {
    fn into_socket_addr(self) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], self))
    }
}

impl IntoSocketAddr for ([u8; 4], u16) {
    fn into_socket_addr(self) -> SocketAddr {
        SocketAddr::from(self)
    }
}

pub struct Server(Router);
impl Server {
    pub fn new() -> Self {
        Server(Router::new())
    }

    pub fn route<T: Endpoint + 'static>(mut self, route: T) -> Self {
        self.0 = self.0.route(Arc::new(route));
        self
    }

    pub fn routes<const SIZE: usize>(mut self, routes: [Arc<dyn Endpoint>; SIZE]) -> Self {
        for route in routes {
            self.0 = self.0.route(route);
        }
        self
    }

    pub fn catches<const SIZE: usize>(mut self, catches: [Arc<dyn Catch>; SIZE]) -> Self {
        for catch in catches {
            self.0 = self.0.catch(catch);
        }
        self
    }

    pub fn catch<T: Catch + 'static>(mut self, catch: T) -> Self {
        self.0 = self.0.catch(Arc::new(catch));
        self
    }

    pub async fn serve<ADDR: IntoSocketAddr>(
        &mut self,
        addr: ADDR,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let addr: SocketAddr = addr.into_socket_addr();

        let listener = TcpListener::bind(addr.clone()).await?;
        println!("Server started at https://{}", addr);

        self.0.serve_routes();

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            let rh = self.0.clone();

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(|req| rh.parse(req)))
                    .await
                {
                    println!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}
