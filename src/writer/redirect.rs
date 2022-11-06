use hyper::{header::LOCATION, http::HeaderValue, StatusCode, Uri};

use crate::error::Error;

use super::Piece;

pub struct Redirect {
    status_code: StatusCode,
    location: HeaderValue,
}

impl Redirect {
    pub fn other(uri: impl TryInto<Uri>) -> Self {
        Self::with_status_code(StatusCode::SEE_OTHER, uri).expect("invalid uri")
    }
    pub fn temporary(uri: impl TryInto<Uri>) -> Self {
        Self::with_status_code(StatusCode::TEMPORARY_REDIRECT, uri).expect("invalid uri")
    }
    pub fn permanent(uri: impl TryInto<Uri>) -> Self {
        Self::with_status_code(StatusCode::PERMANENT_REDIRECT, uri).expect("invalid uri")
    }
    pub fn found(uri: impl TryInto<Uri>) -> Self {
        Self::with_status_code(StatusCode::FOUND, uri).expect("invalid uri")
    }
    pub fn with_status_code(
        status_code: StatusCode,
        uri: impl TryInto<Uri>,
    ) -> Result<Self, Error> {
        if !status_code.is_redirection() {
            return Err(Error::other("not a redirection status code"));
        }

        Ok(Self {
            status_code,
            location: uri
                .try_into()
                .map_err(|_| Error::other("It isn't a valid URI"))
                .and_then(|uri: Uri| {
                    HeaderValue::try_from(uri.to_string())
                        .map_err(|_| Error::other("It isn't a valid header value"))
                })?,
        })
    }
}

impl Piece for Redirect {
    fn render(self, res: &mut crate::http::response::Response) {
        let Self {
            status_code,
            location,
        } = self;
        res.set_status_code(status_code);
        res.headers_mut().insert(LOCATION, location);
    }
}
