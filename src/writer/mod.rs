use async_trait::async_trait;

use crate::{
    depot::Depot,
    http::{request::Request, response::Response},
};

#[async_trait]
pub trait Writer {
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
