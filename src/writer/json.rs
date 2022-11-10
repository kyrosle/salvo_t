use async_trait::async_trait;
use hyper::{header::CONTENT_TYPE, http::HeaderValue};
use serde::Serialize;

use crate::http::errors::StatusError;

use super::Piece;

pub struct Json<T>(pub T);
#[async_trait]
impl<T> Piece for Json<T>
where
    T: Serialize + Send,
{
    fn render(self, res: &mut crate::http::response::Response) {
        match serde_json::to_vec(&self.0) {
            Ok(bytes) => {
                res.headers_mut().insert(
                    CONTENT_TYPE,
                    HeaderValue::from_static("application/json;charset=UTF-8"),
                );
                res.write_body(bytes).ok();
            }
            Err(e) => {
                tracing::error!(error = ?e, "JsonContent write error");
                res.set_status_error(StatusError::internal_server_error());
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use super::*;
    use crate::test::{ResponseExt, TestClient};

    #[tokio::test]
    #[ignore]
    async fn test_write_json_content() {
        #[derive(Serialize, Debug)]
        struct User {
            name: String,
        }
        #[handler(internal)]
        async fn test() -> Json<User> {
            Json(User { name: "jobs".into() })
        }

        let router = Router::new().push(Router::with_path("test").get(test));
        let mut res = TestClient::get("http://127.0.0.1:7878/test").send(router).await;
        assert_eq!(res.take_string().await.unwrap(), r#"{"name":"jobs"}"#);
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "application/json; charset=utf-8"
        );
    }
}
