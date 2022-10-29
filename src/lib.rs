pub mod addr;
pub mod depot;
pub mod http;
pub mod test;
pub mod serde;
pub mod error;
pub mod writer;
pub mod extract;

pub mod handler;
pub mod routing;

use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;