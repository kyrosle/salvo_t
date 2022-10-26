use std::{collections::HashMap, fmt};

use cookie::{Cookie, CookieJar};
use hyper::{
    header::{AsHeaderName, IntoHeaderName},
    http::{Extensions, HeaderValue},
    Body as ReqBody, HeaderMap, Method, Uri, Version,
};
use multimap::MultiMap;
use once_cell::sync::OnceCell;
use serde::Deserialize;

use crate::http::Mime;

use crate::{addr::SocketAddr, error::Error};

use crate::serde::{from_str_multi_val, from_str_val};

use super::errors::ParseError;

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
    // TODO: request add form_data
    // pub(crate) form_data: tokio::sync::OnceCell<FormData>,
    pub(crate) payload: tokio::sync::OnceCell<Vec<u8>>,

    /// Http protocol version
    version: Version,
    pub(crate) remote_addr: Option<SocketAddr>,
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // f.debug_struct("Request").field("method", self.method())
        Ok(())
    }
}

impl Request {
    #[allow(clippy::new_without_default)]
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
    pub fn body(&self) -> Option<&ReqBody> {
        self.body.as_ref()
    }
    pub fn body_mut(&mut self) -> Option<&mut ReqBody> {
        self.body.as_mut()
    }
    pub fn body_take(&mut self) -> Option<ReqBody> {
        self.body.take()
    }
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }
    pub fn accept(&self) -> Vec<Mime> {
        let mut list: Vec<Mime> = vec![];
        if let Some(accept) = self.headers.get("accept").and_then(|h| h.to_str().ok()) {
            let parts: Vec<&str> = accept.split(',').collect();
            for part in parts {
                if let Ok(mt) = part.parse() {
                    list.push(mt);
                }
            }
        }
        list
    }
    pub fn first_accept(&self) -> Option<Mime> {
        let mut accept = self.accept();
        if !accept.is_empty() {
            Some(accept.remove(0))
        } else {
            None
        }
    }
    pub fn content_type(&self) -> Option<Mime> {
        self.headers
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .and_then(|v| v.parse().ok())
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
    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }
    pub fn params_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.params
    }
    pub fn param<'de, T>(&'de self, key: &str) -> Option<T>
    where
        T: Deserialize<'de>,
    {
        self.params.get(key).and_then(|v| from_str_val(v).ok())
    }
    pub fn queries(&self) -> &MultiMap<String, String> {
        self.queries.get_or_init(|| {
            form_urlencoded::parse(self.uri.query().unwrap_or_default().as_bytes())
                .into_owned()
                .collect()
        })
    }
    pub fn query<'de, T>(&'de self, key: &str) -> Option<T>
    where
        T: Deserialize<'de>,
    {
        self.queries()
            .get_vec(key)
            .and_then(|vs| from_str_multi_val(vs).ok())
    }
    pub async fn form<'de, T>(&'de self, key: &str) -> Option<T>
    where
        T: Deserialize<'de>,
    {
        todo!()
    }
    pub async fn form_or_query<'de, T>(&'de self, key: &str) -> Option<T>
    where
        T: Deserialize<'de>,
    {
        todo!()
    }
    pub async fn query_or_form<'de, T>(&'de mut self, key: &str) -> Option<T>
    where
        T: Deserialize<'de>,
    {
        todo!()
    }
    pub async fn file<'a>(&'a mut self, key: &'a str) -> Option<&'a str> {
        todo!()
    }
    pub async fn fist_file(&mut self) -> Option<()> {
        // &FilePart
        todo!()
    }
    pub async fn files<'a>(&'a mut self, key: &'a str) -> Option<()> {
        // &'a Vec<FilePart>
        todo!()
    }
    pub async fn all_files(&mut self) -> Vec<()> {
        // &FilePart
        todo!()
    }
    pub async fn payload(&mut self) -> Result<&Vec<u8>, ParseError> {
        let body = self.body.take();
        self.payload
            .get_or_try_init(|| async {
                match body {
                    Some(body) => hyper::body::to_bytes(body)
                        .await
                        .map(|d| d.to_vec())
                        .map_err(ParseError::Hyper),
                    None => Err(ParseError::EmptyBody),
                }
            })
            .await
    }
    // &FormData
    pub async fn form_data(&mut self) -> Result<(), ParseError> {
        todo!()
    }
    pub async  fn extract<'de, T>(&'de mut self) -> Result<T, ParseError> 
    where T: Extractible<'de>
    {

    }
}
