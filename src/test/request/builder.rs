use std::{borrow::Borrow, sync::Arc};

use async_trait::async_trait;
use hyper::{
    header::{self, IntoHeaderName},
    http::HeaderValue,
    Body, HeaderMap, Method,
};

use url::Url;

use crate::{
    depot::Depot,
    error::Error,
    handler::Handler,
    http::{request::Request, response::Response},
    routing::{router::Router, FlowCtrl},
    service::Service,
};

pub struct RequestBuilder {
    url: Url,
    method: Method,
    headers: HeaderMap,
    // params: HeaderMap,
    body: Body,
}

impl RequestBuilder {
    pub fn new<U>(url: U, method: Method) -> Self
    where
        U: AsRef<str>,
    {
        let url = Url::parse(url.as_ref()).unwrap();
        Self {
            url,
            method,
            headers: HeaderMap::new(),
            // params: HeaderMap::new(),
            body: Body::default(),
        }
    }

    pub fn query<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: ToString,
    {
        self.url
            .query_pairs_mut()
            .append_pair(key.as_ref(), &value.to_string());
        self
    }

    pub fn queries<P, K, V>(mut self, pairs: P) -> Self
    where
        P: IntoIterator,
        P::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: ToString,
    {
        for pair in pairs.into_iter() {
            let (key, value) = pair.borrow();
            self.url
                .query_pairs_mut()
                .append_pair(key.as_ref(), &value.to_string());
        }
        self
    }

    pub fn basic_auth(
        self,
        username: impl std::fmt::Display,
        password: Option<impl std::fmt::Display>,
    ) -> Self {
        let auth = match password {
            Some(password) => format!("{}:{}", username, password),
            None => format!("{}", username),
        };
        let mut encoded = String::from("Basic ");
        base64::encode_config_buf(auth.as_bytes(), base64::STANDARD, &mut encoded);
        self.add_header(header::AUTHORIZATION, encoded, true)
    }

    pub fn bearer_auth(self, token: impl Into<String>) -> Self {
        self.add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", token.into()),
            true,
        )
    }

    pub fn body(mut self, body: impl Into<Body>) -> Self {
        self.body = body.into();
        self
    }

    pub fn text(mut self, body: impl Into<String>) -> Self {
        self.headers
            .entry(header::CONTENT_TYPE)
            .or_insert(HeaderValue::from_static("text/plain; charset=utf-8"));
        self.body(body.into())
    }

    pub fn bytes(mut self, body: Vec<u8>) -> Self {
        self.headers
            .entry(header::CONTENT_TYPE)
            .or_insert(HeaderValue::from_static("application/octet-stream"));

        self.body(body)
    }

    pub fn json<T: serde::Serialize>(mut self, value: &T) -> Self {
        self.headers
            .entry(header::CONTENT_TYPE)
            .or_insert(HeaderValue::from_static("application/json; charset=utf-8"));
        self.body(serde_json::to_vec(value).unwrap())
    }

    pub fn raw_json(mut self, value: impl Into<String>) -> Self {
        self.headers
            .entry(header::CONTENT_TYPE)
            .or_insert(HeaderValue::from_static("application/json; charset=utf-8"));
        self.body(value.into())
    }

    pub fn form<T: serde::Serialize>(mut self, value: &T) -> Self {
        let body = serde_urlencoded::to_string(value).unwrap().into_bytes();
        self.headers
            .entry(header::CONTENT_TYPE)
            .or_insert(HeaderValue::from_static(
                "application/x-www-form-urlencoded",
            ));
        self.body(body)
    }

    pub fn raw_form(mut self, value: impl Into<String>) -> Self {
        self.headers
            .entry(header::CONTENT_TYPE)
            .or_insert(HeaderValue::from_static(
                "application/x-www-form-urlencoded",
            ));
        self.body(value.into())
    }

    pub fn add_header<N, V>(mut self, name: N, value: V, overwrite: bool) -> Self
    where
        N: IntoHeaderName,
        V: TryInto<HeaderValue>,
    {
        let value = value
            .try_into()
            .map_err(|_| Error::Other("invalid header value".into()))
            .unwrap();

        if overwrite {
            self.headers.insert(name, value);
        } else {
            self.headers.append(name, value);
        }
        self
    }

    pub fn build(self) -> Request {
        self.builder_hyper().into()
    }

    pub fn builder_hyper(self) -> hyper::Request<Body> {
        let Self {
            url,
            method,
            headers,
            body,
        } = self;

        let mut req = hyper::Request::builder()
            .method(method)
            .uri(url.to_string());
        (*req.headers_mut().unwrap()) = headers;
        req.body(body).unwrap()
    }

    pub async fn send(self, target: impl SendTarget) -> Response {
        let mut response = target.call(self.build()).await;
        {
            let values = response
                .cookies
                .delta()
                .filter_map(|c| c.encoded().to_string().parse().ok())
                .collect::<Vec<_>>();
            for hv in values {
                response.headers_mut().insert(header::SET_COOKIE, hv);
            }
        }
        response
    }
}

#[async_trait]
pub trait SendTarget {
    async fn call(self, req: Request) -> Response;
}

#[async_trait]
impl SendTarget for &Service {
    async fn call(self, req: Request) -> Response {
        self.handler(req).await
    }
}

#[async_trait]
impl SendTarget for Router {
    async fn call(self, req: Request) -> Response {
        let router = Arc::new(self);
        SendTarget::call(router, req).await
    }
}

#[async_trait]
impl SendTarget for Arc<Router> {
    async fn call(self, req: Request) -> Response {
        let srv = Service::new(self);
        srv.handler(req).await
    }
}

#[async_trait]
impl<T> SendTarget for Arc<T>
where
    T: Handler + Send,
{
    async fn call(self, req: Request) -> Response {
        let mut req = req;
        let mut depot = Depot::new();
        let mut res = Response::with_cookies(req.cookies.clone());
        let mut ctrl = FlowCtrl::new(vec![self.clone()]);
        self.handle(&mut req, &mut depot, &mut res, &mut ctrl).await;
        res
    }
}

#[async_trait]
impl<T> SendTarget for T
where
    T: Handler + Send,
{
    async fn call(self, req: Request) -> Response {
        let handler = Arc::new(self);
        SendTarget::call(handler, req).await
    }
}
