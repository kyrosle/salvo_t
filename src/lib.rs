pub use async_trait::async_trait;
pub use hyper;
pub use salvo_macros::handler;

pub use salvo_macros as macros;

#[macro_use]
mod cfg;

pub mod addr;
pub mod catcher;
mod depot;
mod error;
pub mod extract;
pub mod handler;
pub mod http;
pub mod listener;
pub mod routing;
pub(crate) mod serde;
mod server;
mod service;
pub mod test;
mod transport;
pub mod writer;

pub use self::catcher::{Catcher, CatcherImpl};
pub use self::depot::Depot;
pub use self::error::{BoxedError, Error};
pub use self::extract::Extractible;
pub use self::handler::Handler;
pub use self::http::{Request, Response};
pub use self::listener::Listener;
pub use self::routing::{FlowCtrl, Router};
pub use self::server::Server;
pub use self::service::Service;
pub use self::writer::{Piece, Writer};
/// Result type which has salvo::Error as it's error type.
pub type Result<T> = std::result::Result<T, Error>;

pub mod prelude {
    pub use async_trait::async_trait;
    pub use salvo_macros::{handler, Extractible};

    pub use crate::depot::Depot;
    pub use crate::http::{Request, Response, StatusCode, StatusError};

    pub use crate::extract::LazyExtract;
    pub use crate::handler::{empty_handler, Handler};
    pub use crate::listener::{JoinedListener, Listener, TcpListener};
    pub use crate::routing::{FlowCtrl, Router};
    pub use crate::server::Server;
    pub use crate::service::Service;
    pub use crate::writer::{Json, Piece, Redirect, Text, Writer};
}
pub mod __private {
    pub use once_cell;
    pub use tracing;
}

pub trait PrintSelf
where
    Self: std::fmt::Debug + Sized,
{
    fn print_self(self) -> Self {
        println!("{:#?}", self);
        self
    }
}
