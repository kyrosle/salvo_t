use async_trait::async_trait;

use crate::{
    depot::Depot,
    http::{request::Request, response::Response},
    routing::FlowCtrl,
};

#[async_trait]
pub trait Handler: Send + Sync + 'static {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    );
}
