use hyper::Method;

use super::request::RequestBuilder;

#[derive(Debug, Default)]
pub struct TestClient;

impl TestClient {
    pub fn get(url: impl AsRef<str>) -> RequestBuilder {
        RequestBuilder::new(url, Method::GET)
    }
    pub fn post(url: impl AsRef<str>) -> RequestBuilder {
        RequestBuilder::new(url, Method::POST)
    }
    pub fn put(url: impl AsRef<str>) -> RequestBuilder {
        RequestBuilder::new(url, Method::PUT)
    }
    pub fn delete(url: impl AsRef<str>) -> RequestBuilder {
        RequestBuilder::new(url, Method::DELETE)
    }
    pub fn head(url: impl AsRef<str>) -> RequestBuilder {
        RequestBuilder::new(url, Method::HEAD)
    }
    pub fn options(url: impl AsRef<str>) -> RequestBuilder {
        RequestBuilder::new(url, Method::OPTIONS)
    }
    pub fn patch(url: impl AsRef<str>) -> RequestBuilder {
        RequestBuilder::new(url, Method::PATCH)
    }
    pub fn trace(url: impl AsRef<str>) -> RequestBuilder {
        RequestBuilder::new(url, Method::TRACE)
    }
}