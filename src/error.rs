use async_trait::async_trait;

use crate::{
    depot::Depot,
    http::{
        errors::{ParseError, StatusError},
        request::Request,
        response::Response,
    },
    writer::Writer,
};
use std::{convert::Infallible, error::Error as StdError, fmt::Display, io::Error as IoError};

pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Hyper(hyper::Error),
    HttpParse(ParseError),
    HttpStatus(StatusError),
    Io(IoError),
    SerdeJson(serde_json::Error),
    Anyhow(anyhow::Error),
    Other(BoxedError),
}

impl Error {
    pub fn other(error: impl Into<BoxedError>) -> Self {
        Self::Other(error.into())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hyper(e) => Display::fmt(e, f),
            Self::HttpParse(e) => Display::fmt(e, f),
            Self::HttpStatus(e) => Display::fmt(e, f),
            Self::Io(e) => Display::fmt(e, f),
            Self::SerdeJson(e) => Display::fmt(e, f),
            Self::Anyhow(e) => Display::fmt(e, f),
            Self::Other(e) => Display::fmt(e, f),
        }
    }
}

impl StdError for Error {}

impl From<Infallible> for Error {
    #[inline]
    fn from(infallible: Infallible) -> Error {
        match infallible {}
    }
}
impl From<hyper::Error> for Error {
    #[inline]
    fn from(err: hyper::Error) -> Error {
        Error::Hyper(err)
    }
}
impl From<ParseError> for Error {
    #[inline]
    fn from(err: ParseError) -> Error {
        Error::HttpParse(err)
    }
}
impl From<StatusError> for Error {
    #[inline]
    fn from(err: StatusError) -> Error {
        Error::HttpStatus(err)
    }
}
impl From<IoError> for Error {
    #[inline]
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}
impl From<serde_json::Error> for Error {
    #[inline]
    fn from(err: serde_json::Error) -> Error {
        Error::SerdeJson(err)
    }
}
impl From<anyhow::Error> for Error {
    #[inline]
    fn from(err: anyhow::Error) -> Error {
        Error::Anyhow(err)
    }
}
impl From<BoxedError> for Error {
    #[inline]
    fn from(err: BoxedError) -> Error {
        Error::Other(err)
    }
}

#[async_trait]
impl Writer for Error {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let status_error = match self {
            Error::HttpStatus(e) => e,
            _ => StatusError::internal_server_error(),
        };
        res.set_status_error(status_error);
    }
}

#[async_trait]
impl Writer for anyhow::Error {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.set_status_error(StatusError::internal_server_error());
    }
}

#[cfg(test)]
mod tests {
    use crate::http::*;

    use super::*;

    #[tokio::test]
    async fn test_anyhow() {
        let mut req = Request::default();
        let mut res = Response::default();
        let mut depot = Depot::new();
        let e: anyhow::Error = anyhow::anyhow!("detail message");
        e.write(&mut req, &mut depot, &mut res).await;
        assert_eq!(res.status_code(), Some(StatusCode::INTERNAL_SERVER_ERROR));
    }

    #[tokio::test]
    async fn test_error() {
        let mut req = Request::default();
        let mut res = Response::default();
        let mut depot = Depot::new();

        let e = Error::Other("detail message".into());
        e.write(&mut req, &mut depot, &mut res).await;
        assert_eq!(res.status_code(), Some(StatusCode::INTERNAL_SERVER_ERROR));
    }
}
