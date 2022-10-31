use std::sync::Arc;

use hyper::http::uri::Scheme;

use crate::{handler::Handler, http::request::Request};

use super::{
    filter::{self, Filter},
    PathState,
};

pub struct Router {
    pub(crate) routers: Vec<Router>,
    pub(crate) filters: Vec<Box<dyn Filter>>,
    pub(crate) hoops: Vec<Arc<dyn Handler>>,
    pub(crate) handler: Option<Arc<dyn Handler>>,
}

pub struct DetectMatched {
    pub hoops: Vec<Arc<dyn Handler>>,
    pub handler: Arc<dyn Handler>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    pub fn new() -> Self {
        Self {
            routers: Vec::new(),
            filters: Vec::new(),
            hoops: Vec::new(),
            handler: None,
        }
    }

    pub fn routers(&self) -> &Vec<Router> {
        &self.routers
    }
    pub fn routers_mut(&mut self) -> &mut Vec<Router> {
        &mut self.routers
    }

    pub fn hoops(&self) -> &Vec<Arc<dyn Handler>> {
        &self.hoops
    }
    pub fn hoops_mut(&mut self) -> &mut Vec<Arc<dyn Handler>> {
        &mut self.hoops
    }

    pub fn filters(&self) -> &Vec<Box<dyn Filter>> {
        &self.filters
    }
    pub fn filters_mut(&mut self) -> &mut Vec<Box<dyn Filter>> {
        &mut self.filters
    }

    pub fn detect(&self, req: &mut Request, path_state: &mut PathState) -> Option<DetectMatched> {
        for filter in &self.filters {
            if !filter.filter(req, path_state) {
                return None;
            }
        }

        if !self.routers.is_empty() {
            let original_cursor = path_state.cursor;
            for child in &self.routers {
                if let Some(dm) = child.detect(req, path_state) {
                    return Some(DetectMatched {
                        hoops: [&self.hoops[..], &dm.hoops[..]].concat(),
                        handler: dm.handler.clone(),
                    });
                } else {
                    path_state.cursor = original_cursor;
                }
            }

            if let Some(handler) = self.handler.clone() {
                if path_state.ended() {
                    return Some(DetectMatched {
                        hoops: self.hoops.clone(),
                        handler: handler.clone(),
                    });
                }
            }
        }
        None
    }

    pub fn push(mut self, router: Router) -> Self {
        self.routers.push(router);
        self
    }
    pub fn append(mut self, others: Vec<Router>) -> Self {
        let mut others = others;
        self.routers.append(&mut others);
        self
    }

    pub fn with_hoop<H: Handler>(handler: H) -> Self {
        Router::new().hoop(handler)
    }
    pub fn hoop<H: Handler>(mut self, handler: H) -> Self {
        self.hoops.push(Arc::new(handler));
        self
    }

    pub fn with_path(path: impl Into<String>) -> Self {
        todo!()
    }
    pub fn path(self, path: impl Into<String>) -> Self {
        todo!()
    }

    pub fn with_filter(filter: impl Filter + Sized) -> Self {
        Router::new().filter(filter)
    }
    pub fn filter(mut self, filter: impl Filter + Sized) -> Self {
        self.filters.push(Box::new(filter));
        self
    }
    pub fn with_filter_fn<T>(func: T) -> Self
    where
        T: Fn(&mut Request, &mut PathState) -> bool + Send + Sync + 'static,
    {
        todo!()
    }
    pub fn filter_fn<T>(func: T) -> Self
    where
        T: Fn(&mut Request, &mut PathState) -> bool + Send + Sync + 'static,
    {
        todo!()
    }

    pub fn handle<H: Handler>(mut self, handler: H) -> Self {
        self.handler = Some(Arc::new(handler));
        self
    }
    pub fn then<F>(self, func: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        func(self)
    }
    pub fn scheme(self, scheme: Scheme, default: bool) -> Self {
        self.filter(filter::scheme(scheme, default))
    }
    pub fn host(self, host: impl Into<String>, default: bool) -> Self {
        self.filter(filter::host(host, default))
    }
}
