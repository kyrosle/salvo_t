use std::collections::HashMap;
use std::fmt::{self, Formatter};

use cookie::{Cookie, CookieJar};
use http::header::{self, AsHeaderName, HeaderMap, HeaderValue, IntoHeaderName};
use http::method::Method;
pub use http::request::Parts;
use hyper::http::version::Version;
use hyper::http::{self, Extensions, Uri};
pub use hyper::Body as ReqBody;
use multimap::MultiMap;
use once_cell::sync::OnceCell;
use serde::de::Deserialize;

use crate::addr::SocketAddr;
use crate::extract::{Extractible, Metadata};
use crate::http::form::{FilePart, FormData};
use crate::http::{Mime, ParseError};
use crate::serde::{
    from_request, from_str_map, from_str_multi_map, from_str_multi_val, from_str_val,
};
use crate::Error;

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
    pub(crate) form_data: tokio::sync::OnceCell<FormData>,
    pub(crate) payload: tokio::sync::OnceCell<Vec<u8>>,

    /// Http protocol version
    version: Version,
    pub(crate) remote_addr: Option<SocketAddr>,
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request")
            .field("method", self.method())
            .field("uri", self.uri())
            .field("version", &self.version())
            .field("headers", self.headers())
            // omits Extensions because not useful
            .field("body", &self.body())
            .finish()
    }
}

impl Default for Request {
    fn default() -> Self {
        Request::new()
    }
}

impl From<hyper::Request<ReqBody>> for Request {
    fn from(req: hyper::Request<ReqBody>) -> Self {
        let (
            hyper::http::request::Parts {
                method,
                uri,
                version,
                headers,
                extensions,
                ..
            },
            body,
        ) = req.into_parts();

        let cookies = if let Some(header) = headers.get("Cookie") {
            let mut cookie_jar = CookieJar::new();
            if let Ok(header) = header.to_str() {
                for cookie_str in header.split(';').map(|s| s.trim()) {
                    if let Ok(cookie) = Cookie::parse_encoded(cookie_str).map(|c| c.into_owned()) {
                        cookie_jar.add_original(cookie);
                    }
                }
            }
            cookie_jar
        } else {
            CookieJar::new()
        };

        Request {
            uri,
            headers,
            body: Some(body),
            extensions,
            method,
            cookies,
            queries: OnceCell::new(),
            params: HashMap::new(),
            form_data: tokio::sync::OnceCell::new(),
            payload: tokio::sync::OnceCell::new(),
            version,
            remote_addr: None,
        }
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
            form_data: tokio::sync::OnceCell::new(),
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
    pub fn version(&self) -> Version {
        self.version
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
    pub async fn form<'de, T>(&'de mut self, key: &str) -> Option<T>
    where
        T: Deserialize<'de>,
    {
        self.form_data()
            .await
            .ok()
            .and_then(|ps| ps.fields.get_vec(key))
            .and_then(|vs| from_str_multi_val(vs).ok())
    }
    pub async fn form_or_query<'de, T>(&'de mut self, key: &str) -> Option<T>
    where
        T: Deserialize<'de>,
    {
        if let Ok(form_data) = self.form_data().await {
            if form_data.fields.contains_key(key) {
                return self.form(key).await;
            }
        }
        self.query(key)
    }
    pub async fn query_or_form<'de, T>(&'de mut self, key: &str) -> Option<T>
    where
        T: Deserialize<'de>,
    {
        if self.queries().contains_key(key) {
            self.query(key)
        } else {
            self.form(key).await
        }
    }
    pub async fn file<'a>(&'a mut self, key: &'a str) -> Option<&'a FilePart> {
        self.form_data().await.ok().and_then(|ps| ps.files.get(key))
    }
    pub async fn fist_file(&mut self) -> Option<&FilePart> {
        self.form_data()
            .await
            .ok()
            .and_then(|ps| ps.files.iter().next())
            .map(|(_, f)| f)
    }
    pub async fn files<'a>(&'a mut self, key: &'a str) -> Option<&'a Vec<FilePart>> {
        self.form_data()
            .await
            .ok()
            .and_then(|ps| ps.files.get_vec(key))
    }
    pub async fn all_files(&mut self) -> Vec<&FilePart> {
        self.form_data()
            .await
            .ok()
            .map(|ps| ps.files.iter().map(|(_, f)| f).collect())
            .unwrap_or_default()
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
    pub async fn form_data(&mut self) -> Result<&FormData, ParseError> {
        let ctype = self
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();

        if ctype == "application/x-www-form-urlencoded" || ctype.starts_with("multipart/") {
            let body = self.body.take();
            let headers = self.headers();
            self.form_data
                .get_or_try_init(|| async {
                    match body {
                        Some(body) => FormData::read(headers, body).await,
                        None => Err(ParseError::EmptyBody),
                    }
                })
                .await
        } else {
            Err(ParseError::NotFormData)
        }
    }
    pub async fn extract<'de, T>(&'de mut self) -> Result<T, ParseError>
    where
        T: Extractible<'de>,
    {
        self.extract_with_metadata(T::metadata()).await
    }
    pub async fn extract_with_metadata<'de, T>(
        &'de mut self,
        metadata: &'de Metadata,
    ) -> Result<T, ParseError>
    where
        T: Deserialize<'de>,
    {
        from_request(self, metadata).await
    }
    pub fn parse_params<'de, T>(&'de mut self) -> Result<T, ParseError>
    where
        T: Deserialize<'de>,
    {
        let params = self.params().iter();
        from_str_map(params).map_err(ParseError::Deserialize)
    }
    pub fn parse_queries<'de, T>(&'de mut self) -> Result<T, ParseError>
    where
        T: Deserialize<'de>,
    {
        let queries = self.queries().iter_all();
        from_str_multi_map(queries).map_err(ParseError::Deserialize)
    }
    pub fn parse_headers<'de, T>(&'de mut self) -> Result<T, ParseError>
    where
        T: Deserialize<'de>,
    {
        let iter = self
            .headers()
            .iter()
            .map(|(k, v)| (k.as_str(), v.to_str().unwrap_or_default()));
        from_str_map(iter).map_err(ParseError::Deserialize)
    }
    pub fn parse_cookies<'de, T>(&'de mut self) -> Result<T, ParseError>
    where
        T: Deserialize<'de>,
    {
        let iter = self.cookies().iter().map(|c| c.name_value());
        from_str_map(iter).map_err(ParseError::Deserialize)
    }
    pub async fn parse_json<'de, T>(&'de mut self) -> Result<T, ParseError>
    where
        T: Deserialize<'de>,
    {
        if let Some(ctype) = self.content_type() {
            if ctype.subtype() == mime::JSON {
                return self.payload().await.and_then(|payload| {
                    serde_json::from_slice(payload).map_err(ParseError::SerdeJson)
                });
            }
        }
        Err(ParseError::InvalidContentType)
    }
    pub async fn parse_form<'de, T>(&'de mut self) -> Result<T, ParseError>
    where
        T: Deserialize<'de>,
    {
        if let Some(ctype) = self.content_type() {
            if ctype.subtype() == mime::WWW_FORM_URLENCODED || ctype.subtype() == mime::FORM_DATA {
                return from_str_multi_map(self.form_data().await?.fields.iter_all())
                    .map_err(ParseError::Deserialize);
            }
        }
        Err(ParseError::InvalidContentType)
    }
    pub async fn parse_body<'de, T>(&'de mut self) -> Result<T, ParseError>
    where
        T: Deserialize<'de>,
    {
        if let Some(ctype) = self.content_type() {
            if ctype.subtype() == mime::WWW_FORM_URLENCODED || ctype.subtype() == mime::FORM_DATA {
                return from_str_multi_map(self.form_data().await?.fields.iter_all())
                    .map_err(ParseError::Deserialize);
            } else if ctype.subtype() == mime::JSON {
                return self
                    .payload()
                    .await
                    .and_then(|body| serde_json::from_slice(body).map_err(ParseError::SerdeJson));
            }
        }
        Err(ParseError::InvalidContentType)
    }
}
#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::test::TestClient;

    #[tokio::test]
    async fn test_parse_queries() {
        #[derive(Deserialize, Eq, PartialEq, Debug)]
        struct BadMan<'a> {
            name: &'a str,
            age: u8,
            wives: Vec<String>,
            weapons: (u64, String, String),
        }
        #[derive(Deserialize, Eq, PartialEq, Debug)]
        struct GoodMan {
            name: String,
            age: u8,
            wives: String,
            weapons: u64,
        }
        let mut req = TestClient::get(
            "http://127.0.0.1:7979/hello?name=rust&age=25&wives=a&wives=2&weapons=69&weapons=stick&weapons=gun",
        ).build();
        let man = req.parse_queries::<BadMan>().unwrap();
        assert_eq!(man.name, "rust");
        assert_eq!(man.age, 25);
        assert_eq!(man.wives, vec!["a", "2"]);
        assert_eq!(man.weapons, (69, "stick".into(), "gun".into()));
        let man = req.parse_queries::<GoodMan>().unwrap();
        assert_eq!(man.name, "rust");
        assert_eq!(man.age, 25);
        assert_eq!(man.wives, "a");
        assert_eq!(man.weapons, 69);
    }

    #[tokio::test]
    async fn test_parse_json() {
        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
        struct User {
            name: String,
        }
        let mut req = TestClient::get("http://127.0.0.1:7878/hello")
            .json(&User {
                name: "jobs".into(),
            })
            .build();
        assert_eq!(
            req.parse_json::<User>().await.unwrap(),
            User {
                name: "jobs".into()
            }
        );
    }

    #[tokio::test]
    async fn test_query() {
        let req = TestClient::get(
            "http://127.0.0.1:7979/hello?name=rust&name=25&name=a&name=2&weapons=98&weapons=gun",
        )
        .build();
        assert_eq!(req.queries().len(), 2);
        assert_eq!(req.query::<String>("name").unwrap(), "rust");
        assert_eq!(req.query::<&str>("name").unwrap(), "rust");
        assert_eq!(req.query::<i64>("weapons").unwrap(), 98);
        let names = req.query::<Vec<&str>>("name").unwrap();
        let weapons = req.query::<(u64, &str)>("weapons").unwrap();
        assert_eq!(names, vec!["rust", "25", "a", "2"]);
        assert_eq!(weapons, (98, "gun"));
    }

    #[tokio::test]
    async fn test_form() {
        let mut req = TestClient::post("http://127.0.0.1:7878/hello?q=rust")
            .add_header("content-type", "application/x-www-form-urlencoded", true)
            .raw_form("lover=dog&money=sh*t&q=firefox")
            .build();
        assert_eq!(req.form::<String>("money").await.unwrap(), "sh*t");
        assert_eq!(req.query_or_form::<String>("q").await.unwrap(), "rust");
        assert_eq!(req.form_or_query::<String>("q").await.unwrap(), "firefox");

        let mut req: Request = TestClient::post("http://127.0.0.1:7878/hello?q=rust")
            .add_header(
                "content-type",
                "multipart/form-data; boundary=----WebKitFormBoundary0mkL0yrNNupCojyz",
                true,
            )
            .body(
                "------WebKitFormBoundary0mkL0yrNNupCojyz\r\n\
Content-Disposition: form-data; name=\"money\"\r\n\r\nsh*t\r\n\
------WebKitFormBoundary0mkL0yrNNupCojyz\r\n\
Content-Disposition: form-data; name=\"file1\"; filename=\"err.txt\"\r\n\
Content-Type: text/plain\r\n\r\n\
file content\r\n\
------WebKitFormBoundary0mkL0yrNNupCojyz--\r\n",
            )
            .build();
        assert_eq!(req.form::<String>("money").await.unwrap(), "sh*t");
        let file = req.file("file1").await.unwrap();
        assert_eq!(file.name().unwrap(), "err.txt");
        assert_eq!(file.headers().get("content-type").unwrap(), "text/plain");
        let files = req.files("file1").await.unwrap();
        assert_eq!(files[0].name().unwrap(), "err.txt");
    }
}
