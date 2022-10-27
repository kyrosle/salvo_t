use std::{collections::VecDeque, pin::Pin};

use hyper::body::Bytes;
use futures::stream::Stream;
use std::error::Error as  StdError;

#[allow(clippy::type_complexity)]
#[non_exhaustive]
pub enum ResBody {
    None,
    Once(Bytes),
    Chunk(VecDeque<Bytes>),
    Stream(Pin<Box<dyn Stream<Item = Result<Bytes, Box<dyn StdError + Send + Sync>>> + Send >>),
}