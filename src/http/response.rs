use std::{collections::VecDeque, pin::Pin, task::Poll};

use cookie::{Cookie, CookieJar};
use futures::{Stream, TryStreamExt};
use hyper::{
    body::Bytes,
    header::{IntoHeaderName, CONTENT_LENGTH, SET_COOKIE},
    http::HeaderValue,
    HeaderMap, StatusCode, Version,
};

use std::error::Error as StdError;

use crate::{error::Error, writer::Piece, PrintSelf};

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

impl PrintSelf for Response {}

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
    pub fn body(&self) -> &ResBody {
        &self.body
    }
    pub fn body_mut(&mut self) -> &mut ResBody {
        &mut self.body
    }
    pub fn set_body(&mut self, body: ResBody) {
        self.body = body;
    }
    pub fn replace_body(&mut self, body: ResBody) -> ResBody {
        std::mem::replace(&mut self.body, body)
    }
    pub fn take_body(&mut self) -> ResBody {
        std::mem::replace(&mut self.body, ResBody::None)
    }
    pub fn is_stamped(&mut self) -> bool {
        if let Some(code) = self.status_code() {
            if code.is_client_error() || code.is_server_error() || code.is_redirection() {
                return true;
            }
        }
        false
    }
    pub fn write_cookies_to_headers(&mut self) {
        for cookie in self.cookies.delta() {
            if let Ok(hv) = cookie.encoded().to_string().parse() {
                self.headers.append(SET_COOKIE, hv);
            }
        }
        self.cookies = CookieJar::new();
    }
    pub(crate) async fn write_back(mut self, res: &mut hyper::Response<hyper::Body>) {
        self.write_cookies_to_headers();
        let Self {
            status_code,
            headers,
            body,
            ..
        } = self;
        *res.headers_mut() = headers;

        *res.status_mut() = status_code.unwrap_or(StatusCode::NOT_FOUND);

        match body {
            ResBody::None => {
                res.headers_mut()
                    .insert(CONTENT_LENGTH, HeaderValue::from_static("0"));
            }
            ResBody::Once(bytes) => {
                *res.body_mut() = hyper::Body::from(bytes);
            }
            ResBody::Chunks(chunks) => {
                *res.body_mut() = hyper::Body::wrap_stream(tokio_stream::iter(
                    chunks
                        .into_iter()
                        .map(Result::<_, Box<dyn StdError + Sync + Send>>::Ok),
                ));
            }
            ResBody::Stream(stream) => {
                *res.body_mut() = hyper::Body::wrap_stream(stream);
            }
        }
    }
    pub fn cookies(&self) -> &CookieJar {
        &self.cookies
    }
    pub fn cookies_mut(&mut self) -> &mut CookieJar {
        &mut self.cookies
    }
    pub fn cookie<T>(&self, name: T) -> Option<&Cookie<'static>>
    where
        T: AsRef<str>,
    {
        self.cookies.get(name.as_ref())
    }
    pub fn add_cookie(&mut self, cookie: Cookie<'static>) -> &mut Self {
        self.cookies.add(cookie);
        self
    }
    pub fn with_cookie(&mut self, cookie: Cookie<'static>) -> &mut Self {
        self.add_cookie(cookie);
        self
    }
    pub fn remove_cookie(&mut self, name: &str) -> &mut Self {
        if let Some(cookie) = self.cookies.get(name).cloned() {
            self.cookies.remove(cookie);
        }
        self
    }
    pub fn status_code(&self) -> Option<StatusCode> {
        self.status_code
    }
    pub fn set_status_code(&mut self, code: StatusCode) {
        self.status_code = Some(code);
        if !code.is_success() {
            self.status_error = StatusError::from_code(code);
        }
    }
    pub fn with_status_code(&mut self, code: StatusCode) -> &mut Self {
        self.set_status_code(code);
        self
    }
    pub fn status_error(&self) -> Option<&StatusError> {
        self.status_error.as_ref()
    }
    pub fn set_status_error(&mut self, err: StatusError) {
        self.status_code = Some(err.code);
        self.status_error = Some(err);
    }
    pub fn with_status_error(&mut self, err: StatusError) -> &mut Self {
        self.set_status_error(err);
        self
    }

    pub fn render<P>(&mut self, piece: P)
    where
        P: Piece,
    {
        piece.render(self)
    }
    pub fn with_render<P>(&mut self, piece: P) -> &mut Self
    where
        P: Piece,
    {
        self.render(piece);
        self
    }
    pub fn stuff<P>(&mut self, code: StatusCode, piece: P)
    where
        P: Piece,
    {
        self.status_code = Some(code);
        piece.render(self)
    }
    pub fn with_stuff<P>(&mut self, code: StatusCode, piece: P) -> &mut Self
    where
        P: Piece,
    {
        self.stuff(code, piece);
        self
    }

    pub fn write_body(&mut self, data: impl Into<Bytes>) -> crate::Result<()> {
        match self.body_mut() {
            ResBody::None => {
                self.body = ResBody::Once(data.into());
            }
            ResBody::Once(ref bytes) => {
                let mut chunks = VecDeque::new();
                chunks.push_back(bytes.clone());
                chunks.push_back(data.into());
                self.body = ResBody::Chunks(chunks);
            }
            ResBody::Chunks(chunks) => {
                chunks.push_back(data.into());
            }
            ResBody::Stream(_) => {
                tracing::error!(
                    "current body kind is `ResBody::Stream`, try to write byres to it "
                );
                return Err(Error::other(
                    "current body kind is `ResBody::Stream`, try to write byres to it ",
                ));
            }
        }
        Ok(())
    }
    pub fn streaming<S, O, E>(&mut self, stream: S) -> crate::Result<()>
    where
        S: Stream<Item = Result<O, E>> + Send + Sync + 'static,
        O: Into<Bytes> + 'static,
        E: Into<Box<dyn StdError + Send + Sync>> + 'static,
    {
        match &self.body {
            ResBody::Once(_) => {
                return Err(Error::other("current body kind is `ResBody::Once` already"));
            }
            ResBody::Chunks(_) => {
                return Err(Error::other(
                    "current body kind is `ResBody::Chunks` already",
                ));
            }
            ResBody::Stream(_) => {
                return Err(Error::other(
                    "current body kind is `ResBody::Stream` already",
                ));
            }
            _ => {}
        }
        let mapped = stream.map_ok(Into::into).map_err(Into::into);
        self.body = ResBody::Stream(Box::pin(mapped));
        Ok(())
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "HTTP/1.1 {}\n{:?}",
            self.status_code.unwrap_or(StatusCode::NOT_FOUND),
            self.headers
        )
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use futures_util::stream::{iter, StreamExt};
    use std::error::Error;

    use super::*;

    #[test]
    fn test_body_empty() {
        let body = ResBody::Once(Bytes::from("hello"));
        assert!(!body.is_none());
        let body = ResBody::None;
        assert!(body.is_none());
    }

    #[tokio::test]
    async fn test_body_stream1() {
        let mut body = ResBody::Once(Bytes::from("hello"));

        let mut result = BytesMut::new();
        while let Some(Ok(data)) = body.next().await {
            result.extend_from_slice(&data);
        }

        assert_eq!("hello", result);
    }

    #[tokio::test]
    async fn test_body_stream2() {
        let mut body = ResBody::Stream(Box::pin(iter(vec![
            Result::<_, Box<dyn Error + Send + Sync>>::Ok(BytesMut::from("Hello").freeze()),
            Result::<_, Box<dyn Error + Send + Sync>>::Ok(BytesMut::from(" World").freeze()),
        ])));

        let mut result = BytesMut::new();
        while let Some(Ok(data)) = body.next().await {
            result.extend_from_slice(&data);
        }
        assert_eq!("Hello World", &result);
    }
}
