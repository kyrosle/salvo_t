use std::marker::PhantomData;

use serde::Deserialize;

/// Metadata types.
pub mod metadata;
use crate::http::ParseError;
use crate::Request;
pub use metadata::Metadata;
pub trait Extractible<'de>: Deserialize<'de> {
    fn metadata() -> &'de Metadata;
}

#[derive(Deserialize)]
pub struct LazyExtract<T> {
    #[serde(skip)]
    _inner: PhantomData<T>,
}

impl<'de, T: Extractible<'de>> LazyExtract<T> {
    pub fn new() -> Self {
        LazyExtract {
            _inner: PhantomData::<T>,
        }
    }
    pub async fn extract(self, req: &'de mut Request) -> Result<T, ParseError> {
        req.extract().await
    }
}

impl<'de, T> Extractible<'de> for LazyExtract<T>
where
    T: Extractible<'de>,
{
    fn metadata() -> &'de Metadata {
        T::metadata()
    }
}
