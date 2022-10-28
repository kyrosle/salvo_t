use std::{collections::VecDeque, pin::Pin, task::Poll};

use cookie::{Cookie, CookieJar};
use futures::{Stream, TryStreamExt};
use hyper::{
    body::Bytes,
    header::{IntoHeaderName, SET_COOKIE},
    http::HeaderValue,
    HeaderMap, StatusCode, Version,
};

use std::error::Error as StdError;

use crate::error::Error;

use super::errors::StatusError;

#[allow(clippy::type_complexity)]
#[non_exhaustive]
pub enum ResBody {
    /// None body.
    None,
    /// Once bytes body.
    Once(Bytes),
    /// Chunks body.
    Chunks(VecDeque<Bytes>),
    /// Stream body.
    Stream(Pin<Box<dyn Stream<Item = Result<Bytes, Box<dyn StdError + Send + Sync>>> + Send>>),
}

impl ResBody {
    pub fn is_none(&self) -> bool {
        matches!(*self, ResBody::None)
    }
    pub fn is_once(&self) -> bool {
        matches!(*self, ResBody::Once(_))
    }
    pub fn is_chunks(&self) -> bool {
        matches!(*self, ResBody::Chunks(_))
    }
    pub fn is_stream(&self) -> bool {
        matches!(*self, ResBody::Stream(_))
    }
    pub fn size(&self) -> Option<u64> {
        match self {
            ResBody::None => Some(0),
            ResBody::Once(bytes) => Some(bytes.len() as u64),
            ResBody::Chunks(chunks) => Some(chunks.iter().map(|bytes| bytes.len() as u64).sum()),
            ResBody::Stream(_) => None,
        }
    }
}

impl Stream for ResBody {
    type Item = Result<Bytes, Box<dyn StdError + Send + Sync>>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.get_mut() {
            ResBody::None => Poll::Ready(None),
            ResBody::Once(bytes) => {
                if bytes.is_empty() {
                    Poll::Ready(None)
                } else {
                    let bytes = std::mem::replace(bytes, Bytes::new());
                    Poll::Ready(Some(Ok(bytes)))
                }
            }
            ResBody::Chunks(chunks) => Poll::Ready(chunks.pop_front().map(Ok)),
            ResBody::Stream(stream) => stream.as_mut().poll_next(cx),
        }
    }
}

impl From<hyper::Body> for ResBody {
    fn from(hbody: hyper::Body) -> Self {
        ResBody::Stream(Box::pin(
            hbody.map_err(|e| e.into_cause().unwrap()).into_stream(),
        ))
    }
}

pub struct Response {
    status_code: Option<StatusCode>,
    pub(crate) status_error: Option<StatusError>,
    headers: HeaderMap,
    version: Version,
    pub(crate) cookies: CookieJar,
    pub(crate) body: ResBody,
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

impl From<hyper::Response<hyper::Body>> for Response {
    fn from(res: hyper::Response<hyper::Body>) -> Self {
        let (
            hyper::http::response::Parts {
                status,
                version,
                headers,
                ..
            },
            body,
        ) = res.into_parts();
        let cookies = if let Some(header) = headers.get(SET_COOKIE) {
            let mut cookie_jar = CookieJar::new();
            if let Ok(header) = header.to_str() {
                for cookie_str in header.split(';').map(|s| s.trim()) {
                    if let Ok(cookie) = Cookie::parse_encoded(cookie_str).map(|c| c.into_owned()) {
                        cookie_jar.add(cookie);
                    }
                }
            }
            cookie_jar
        } else {
            CookieJar::new()
        };

        Response {
            status_code: Some(status),
            status_error: None,
            body: body.into(),
            version,
            headers,
            cookies,
        }
    }
}

impl Response {
    pub fn new() -> Response {
        Response {
            status_code: None,
            status_error: None,
            body: ResBody::None,
            version: Version::default(),
            headers: HeaderMap::new(),
            cookies: CookieJar::default(),
        }
    }
    pub fn with_cookies(cookies: CookieJar) -> Response {
        Response {
            status_code: None,
            status_error: None,
            body: ResBody::None,
            version: Version::default(),
            headers: HeaderMap::new(),
            cookies,
        }
    }
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }
    pub fn set_headers(&mut self, headers: HeaderMap) {
        self.headers = headers;
    }

    pub fn add_header<N, V>(&mut self, name: N, value: V, overwrite: bool) -> crate::Result<()>
    where
        N: IntoHeaderName,
        V: TryInto<HeaderValue>,
    {
        let value = value
            .try_into()
            .map_err(|_| Error::Other("invalid header value".into()))?;
        if overwrite {
            self.headers.insert(name, value);
        } else {
            self.headers.append(name, value);
        }
        Ok(())
    }
    pub fn with_header<N, V>(
        &mut self,
        name: N,
        value: V,
        overwrite: bool,
    ) -> crate::Result<&mut Self>
    where
        N: IntoHeaderName,
        V: TryInto<HeaderValue>,
    {
        self.add_header(name, value, overwrite)?;
        Ok(self)
    }
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.version
    }
    pub fn set_body(&mut self, body: ResBody) {
        self.body = body;
    }
    pub fn replace_body(&mut self, body: ResBody) -> ResBody {
        std::mem::replace(&mut self.body, body)
    }
}
