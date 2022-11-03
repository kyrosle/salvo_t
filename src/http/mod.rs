pub mod errors;
pub mod form;
pub mod request;
pub mod response;

pub use mime::{self, Mime};

use self::request::Request;

pub(crate) fn guess_accept_mine(req: &Request, default_type: Option<Mime>) -> Mime {
    let dmime: Mime = default_type.unwrap_or_else(|| "text/html".parse().unwrap());
    let accept = req.accept();
    accept
        .first()
        .unwrap_or(&dmime)
        .to_string()
        .parse()
        .unwrap_or(dmime)
}
