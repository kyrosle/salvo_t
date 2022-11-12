use hyper::{header::CONTENT_TYPE, http::HeaderValue};

use crate::http::response::Response;

use super::Piece;

#[non_exhaustive]
pub enum Text<C> {
    /// It will set `content-type` to `text/plain; charset=utf-8`.
    Plain(C),
    /// It will set `content-type` to `application/json; charset=utf-8`.
    Json(C),
    /// It will set `content-type` to `application/xml; charset=utf-8`.
    Xml(C),
    /// It will set `content-type` to `text/html; charset=utf-8`.
    Html(C),
    /// It will set `content-type` to `text/javascript; charset=utf-8`.
    Js(C),
    /// It will set `content-type` to `text/css; charset=utf-8`.
    Css(C),
    /// It will set `content-type` to `text/csv; charset=utf-8`.
    Csv(C),
    /// It will set `content-type` to `application/atom+xml; charset=utf-8`.
    Atom(C),
    /// It will set `content-type` to `application/rss+xml; charset=utf-8`.
    Rss(C),
    /// It will set `content-type` to `application/rdf+xml; charset=utf-8`.
    Rdf(C),
}

impl<C> Text<C>
where
    C: AsRef<str>,
{
    fn set_header(self, res: &mut Response) -> C {
        let (ctype, content) = match self {
            Self::Plain(content) => (
                HeaderValue::from_static("text/plain; charset=utf-8"),
                content,
            ),
            Self::Json(content) => (
                HeaderValue::from_static("application/json; charset=utf-8"),
                content,
            ),
            Self::Xml(content) => (
                HeaderValue::from_static("application/xml; charset=utf-8"),
                content,
            ),
            Self::Html(content) => (
                HeaderValue::from_static("text/html; charset=utf-8"),
                content,
            ),
            Self::Js(content) => (
                HeaderValue::from_static("text/javascript; charset=utf-8"),
                content,
            ),
            Self::Css(content) => (HeaderValue::from_static("text/css; charset=utf-8"), content),
            Self::Csv(content) => (HeaderValue::from_static("text/csv; charset=utf-8"), content),
            Self::Atom(content) => (
                HeaderValue::from_static("application/atom+xml; charset=utf-8"),
                content,
            ),
            Self::Rss(content) => (
                HeaderValue::from_static("application/rss+xml; charset=utf-8"),
                content,
            ),
            Self::Rdf(content) => (
                HeaderValue::from_static("application/rdf+xml; charset=utf-8"),
                content,
            ),
        };
        res.headers_mut().insert(CONTENT_TYPE, ctype);
        content
    }
}

impl Piece for Text<&'static str> {
    fn render(self, res: &mut Response) {
        let content = self.set_header(res);
        res.write_body(content).ok();
    }
}

impl Piece for Text<String> {
    fn render(self, res: &mut Response) {
        let content = self.set_header(res);
        res.write_body(content).ok();
    }
}

impl<'a> Piece for Text<&'a String> {
    fn render(self, res: &mut Response) {
        let content = self.set_header(res);
        res.write_body(content.as_bytes().to_vec()).ok();
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use super::*;
    use crate::test::{ResponseExt, TestClient};

    #[tokio::test]
    async fn test_write_str() {
        #[handler(internal)]
        async fn test() -> &'static str {
            "hello"
        }

        let router = Router::new().push(Router::with_path("test").get(test));

        let mut res = TestClient::get("http://127.0.0.1:7878/test")
            .send(router)
            .await;
        assert_eq!(res.take_string().await.unwrap(), "hello");
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );
    }

    #[tokio::test]
    async fn test_write_string() {
        #[handler(internal)]
        async fn test() -> String {
            "hello".to_owned()
        }

        let router = Router::new().push(Router::with_path("test").get(test));
        let mut res = TestClient::get("http://127.0.0.1:7878/test")
            .send(router)
            .await;
        assert_eq!(res.take_string().await.unwrap(), "hello");
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );
    }

    #[tokio::test]
    async fn test_write_plain_text() {
        #[handler(internal)]
        async fn test() -> Text<&'static str> {
            Text::Plain("hello")
        }

        let router = Router::new().push(Router::with_path("test").get(test));

        let mut res = TestClient::get("http://127.0.0.1:7878/test")
            .send(router)
            .await;
        assert_eq!(res.take_string().await.unwrap(), "hello");
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );
    }

    #[tokio::test]
    async fn test_write_json_text() {
        #[handler(internal)]
        async fn test() -> Text<&'static str> {
            Text::Json(r#"{"hello": "world"}"#)
        }

        let router = Router::new().push(Router::with_path("test").get(test));
        let mut res = TestClient::get("http://127.0.0.1:7878/test")
            .send(router)
            .await;
        assert_eq!(res.take_string().await.unwrap(), r#"{"hello": "world"}"#);
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "application/json; charset=utf-8"
        );
    }

    #[tokio::test]
    async fn test_write_html_text() {
        #[handler(internal)]
        async fn test() -> Text<&'static str> {
            Text::Html("<html><body>hello</body></html>")
        }

        let router = Router::new().push(Router::with_path("test").get(test));
        let mut res = TestClient::get("http://127.0.0.1:7878/test")
            .send(router)
            .await;
        assert_eq!(
            res.take_string().await.unwrap(),
            "<html><body>hello</body></html>"
        );
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );
    }
}
