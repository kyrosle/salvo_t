use std::{collections::HashMap, fmt};

use cookie::CookieJar;
use hyper::{
    header::{AsHeaderName, IntoHeaderName},
    http::{Extensions, HeaderValue},
    Body as ReqBody, HeaderMap, Method, Uri, Version,
};
use multimap::MultiMap;
use once_cell::sync::OnceCell;
use serde::Deserialize;

use crate::{addr::SocketAddr, serde::from_str_multi_val};

pub struct Request {
    uri: Uri,
    headers: HeaderMap,
    body: Option<ReqBody>,
    extensions: Extensions,
    method: Method,
    pub(crate) cookies: CookieJar,
    pub(crate) params: HashMap<String, String>,

    // accept: Option<Vec<Mime>>,
    pub(crate) queries: OnceCell<MultiMap<String, String>>,
    // pub(crate) form_data: tokio::sync::OnceCell<FormData>,
    pub(crate) payload: tokio::sync::OnceCell<Vec<u8>>,

    /// Http protocol version
    version: Version,
    pub(crate) remote_addr: Option<SocketAddr>,
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request").field("method", self.method())
    }
}

impl Request {
    pub fn new() -> Request {
        Request {
            uri: Uri::default(),
            headers: HeaderMap::new(),
            body: Some(ReqBody::default()),
            extensions: Extensions::default(),
            method: Method::default(),
            cookies: CookieJar::default(),
            params: HashMap::new(),
            queries: OnceCell::new(),
            // form_data: tokio::sync::OnceCell::new(),
            payload: tokio::sync::OnceCell::new(),
            version: Version::default(),
            remote_addr: None,
        }
    }
    pub fn uri(&self) -> &Uri {
        &self.uri
    }
    pub fn method(&self) -> &Method {
        &self.method
    }
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }
    pub fn remote_addr(&self) -> Option<&SocketAddr> {
        self.remote_addr.as_ref()
    }
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }
    pub fn header<'de, T>(&'de self, key: impl AsHeaderName) -> Option<T>
    where
        T: Deserialize<'de>,
    {
        let values = self
            .headers
            .get_all(key)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect::<Vec<_>>();
        from_str_multi_val(values).ok()
    }
    pub fn add_header<N, V>(&mut self, name: N, value: V, overwrite: bool) -> crate::Result<()>
    where
        N: IntoHeaderName,
        V: TryInto<HeaderValue>,
    {
        Ok(())
    }
}
