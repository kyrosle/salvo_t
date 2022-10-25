pub mod addr;
pub mod depot;
pub mod http;
pub mod test;
pub mod serde;
mod error;
pub mod writer;


use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;