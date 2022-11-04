pub mod addr;
pub mod depot;
pub mod error;
pub mod extract;
pub mod http;
pub mod serde;
pub mod test;
pub mod writer;

pub mod catcher;
pub mod service;
pub mod transport;

pub mod handler;
pub mod listener;
pub mod routing;

pub mod cfg;

use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;
