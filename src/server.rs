use crate::{listener::Listener, service::Service, transport::Transport};

use core::future::Future;
use hyper::Server as HyperServer;
use std::error::Error as StdError;

pub struct Server<L> {
    listener: L,
}

impl<L> Server<L>
where
    L: Listener,
    L::Conn: Transport + Send + Unpin + 'static,
    L::Error: Into<Box<(dyn StdError + Send + Sync + 'static)>>,
{
    fn new(listener: L) -> Self {
        Server { listener }
    }
    pub async fn serve<S>(self, service: S)
    where
        S: Into<Service>,
    {
        self.try_serve(service).await.unwrap();
    }

    pub async fn try_serve<S>(self, service: S) -> Result<(), hyper::Error>
    where
        S: Into<Service>,
    {
        HyperServer::builder(self.listener)
            .serve(service.into())
            .await
    }

    pub async fn serve_with_graceful_shutdown<S, G>(self, addr: S, signal: G)
    where
        S: Into<Service>,
        G: Future<Output = ()> + Send + 'static,
    {
        self.try_serve_with_graceful_shutdown(addr, signal).await.unwrap();
    }

    pub async fn try_serve_with_graceful_shutdown<S, G>(
        self,
        service: S,
        signal: G,
    ) -> Result<(), hyper::Error>
    where
        S: Into<Service>,
        G: Future<Output = ()> + Send + 'static,
    {
        let server = HyperServer::builder(self.listener).serve(service.into());
        if let Err(err) = server.with_graceful_shutdown(signal).await {
            tracing::error!("server error: {}", err);
            Err(err)
        } else {
            Ok(())
        }
    }
}
