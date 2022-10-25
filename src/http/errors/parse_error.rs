use async_trait::async_trait;
use serde::de::value::Error as DeError;
use std::{io::Error as IoError, str::Utf8Error};
use thiserror::Error;

use crate::writer::Writer;

pub type ParseResult<T> = Result<T, ParseError>;
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ParseError {
    /// The Hyper request did not have a valid Content-Type header.
    #[error("The Hyper request did not have a valid Content-Type header.")]
    InvalidContentType,

    /// The Hyper request's body is empty.
    #[error("The Hyper request's body is empty.")]
    EmptyBody,

    /// Parse error when parse from str.
    #[error("Parse error when parse from str.")]
    ParseFromStr,

    /// Parse error when parse from str.
    #[error("Parse error when decode url.")]
    UrlDecode,

    /// Deserialize error when parse from request.
    #[error("Deserialize error.")]
    Deserialize(#[from] DeError),

    /// DuplicateKey.
    #[error("DuplicateKey.")]
    DuplicateKey,

    /// The Hyper request Content-Type top-level Mime was not `Multipart`.
    #[error("The Hyper request Content-Type top-level Mime was not `Multipart`.")]
    NotMultipart,

    /// The Hyper request Content-Type sub-level Mime was not `FormData`.
    #[error("The Hyper request Content-Type sub-level Mime was not `FormData`.")]
    NotFormData,

    /// InvalidRange.
    #[error("InvalidRange")]
    InvalidRange,

    /// An multer error.
    #[error("Multer error: {0}")]
    Multer(#[from] multer::Error),

    /// An I/O error.
    #[error("I/O error: {}", _0)]
    Io(#[from] IoError),

    /// An error was returned from hyper.
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    /// An error occurred during UTF-8 processing.
    #[error("UTF-8 processing error: {0}")]
    Utf8(#[from] Utf8Error),

    /// Serde json error.
    #[error("Serde json error: {0}")]
    SerdeJson(#[from] serde_json::error::Error),
}

// TODO: impl Writer for ParseError
#[async_trait]
impl Writer for ParseError {
    async fn write(mut self) {

    }
}

// TODO: error - parse error test
#[cfg(test)]
mod test {
    use async_trait::async_trait;
}
