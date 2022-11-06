use hyper::{header::CONTENT_TYPE, http::HeaderValue};
use serde::Serialize;

use crate::http::errors::StatusError;

use super::Piece;

pub struct Json<T>(pub T);

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
