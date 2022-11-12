mod json;
mod redirect;
mod text;

pub use json::Json;
pub use redirect::Redirect;
pub use text::Text;

use crate::http::header::{HeaderValue, CONTENT_TYPE};
use crate::{async_trait, Depot, Request, Response};

#[async_trait]
pub trait Writer {
    #[must_use = "write future must be used"]
    async fn write(mut self, req: &mut Request, depot: &mut Depot, res: &mut Response);
}

#[async_trait]
impl<T, E> Writer for Result<T, E>
where
    T: Writer + Send,
    E: Writer + Send,
{
    async fn write(mut self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        match self {
            Ok(v) => v.write(req, depot, res).await,
            Err(e) => e.write(req, depot, res).await,
        }
    }
}

pub trait Piece {
    fn render(self, res: &mut Response);
}

#[async_trait]
impl<P> Writer for P
where
    P: Piece + Sized + Send,
{
    async fn write(mut self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        self.render(res)
    }
}

impl Piece for () {
    fn render(self, _res: &mut Response) {}
}

impl Piece for &'static str {
    fn render(self, res: &mut Response) {
        res.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        res.write_body(self.as_bytes().to_vec()).ok();
    }
}

impl<'a> Piece for &'a String {
    fn render(self, res: &mut Response) {
        res.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        res.write_body(self.as_bytes().to_vec()).ok();
    }
}

impl Piece for String {
    fn render(self, res: &mut Response) {
        res.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        res.write_body(self.as_bytes().to_vec()).ok();
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

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
}
