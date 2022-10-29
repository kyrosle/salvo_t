use std::sync::Arc;

use crate::{
    depot::Depot,
    handler::Handler,
    http::{request::Request, response::Response},
};

pub struct FlowCtrl {
    is_ceased: bool,
    cursor: usize,
    pub(crate) handlers: Vec<Arc<dyn Handler>>,
}

impl FlowCtrl {
    pub fn new(handlers: Vec<Arc<dyn Handler>>) -> Self {
        FlowCtrl {
            is_ceased: false,
            cursor: 0,
            handlers,
        }
    }
    pub fn has_next(&self) -> bool {
        self.cursor < self.handlers.len() && !self.handlers.is_empty()
    }
    pub async fn call_next(&mut self, req: &mut Request, depot: &mut Depot, res: &mut Response) -> bool {
        if res.is_stamped() {
            self.skip_rest();
            return false;
        }
        let mut handler = self.handlers.get(self.cursor).cloned();
        if handler.is_none() {
            false
        } else {
            while let Some(h) = handler.take() {
                self.cursor += 1;
                h.handle(req, depot, res, self).await;
                if res.is_stamped() {
                    self.skip_rest();
                    return true;
                } else {
                    handler = self.handlers.get(self.cursor).cloned();
                }
            }
            true
        }
    }
    pub fn skip_rest(&mut self) {
        self.cursor = self.handlers.len()
    }
    pub fn is_ceased(&self) -> bool {
        self.is_ceased
    }
    pub fn cease(&mut self) {
        self.skip_rest();
        self.is_ceased = true;
    }
}
