use std::{
    fmt::{self, format, Formatter},
    sync::Arc,
};

use hyper::http::uri::Scheme;

use crate::{handler::Handler, http::request::Request};

use super::{
    filter::{self, Filter, FnFilter, PathFilter},
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

macro_rules! method_server {
    ($($name: ident),*) => {
        $(
            pub fn $name<H: Handler>(self, handler: H) -> Self {
                self.push(Router::with_filter(filter::$name()).handle(handler))
            }
        )*
    };
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
        Router::with_filter(PathFilter::new(path))
    }
    pub fn path(self, path: impl Into<String>) -> Self {
        self.filter(PathFilter::new(path))
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
        Router::with_filter(FnFilter(func))
    }
    pub fn filter_fn<T>(self, func: T) -> Self
    where
        T: Fn(&mut Request, &mut PathState) -> bool + Send + Sync + 'static,
    {
        self.filter(FnFilter(func))
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
    pub fn port(self, port: u16, default: bool) -> Self {
        self.filter(filter::port(port, default))
    }
    method_server!(get, post, put, delete, patch, head, options);
}

const SYMBOL_DOWN: &str = "│";
const SYMBOL_TEE: &str = "├";
const SYMBOL_ELL: &str = "└";
const SYMBOL_RIGHT: &str = "─";

impl fmt::Debug for Router {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fn print(f: &mut Formatter, prefix: &str, last: bool, router: &Router) -> fmt::Result {
            let mut path = "".to_owned();
            let mut others = Vec::with_capacity(router.filters.len());
            if router.filters.is_empty() {
                path = "!NULL!".to_owned();
            } else {
                for filter in &router.filters {
                    let info = format!("{:?}", filter);
                    if info.starts_with("path:") {
                        path = info.split_once(':').unwrap().1.to_owned();
                    } else {
                        let mut parts = info.splitn(2, ':').collect::<Vec<_>>();
                        if !parts.is_empty() {
                            others.push(parts.pop().unwrap().to_owned());
                        }
                    }
                }
            }
            let cp = if last {
                format!("{}{}{}{}", prefix, SYMBOL_ELL, SYMBOL_RIGHT, SYMBOL_RIGHT)
            } else {
                format!("{}{}{}{}", prefix, SYMBOL_TEE, SYMBOL_RIGHT, SYMBOL_RIGHT)
            };
            let hd = if let Some(handler) = &router.handler {
                format!(" -> {}", handler.type_name())
            } else {
                "".into()
            };
            if !others.is_empty() {
                writeln!(f, "{}{}[{}]{}", cp, path, others.join(","), hd)?;
            } else {
                writeln!(f, "{}{}{}", cp, path, hd)?;
            }
            let routers = router.routers();
            if !routers.is_empty() {
                let np = if last {
                    format!("{}    ", prefix)
                } else {
                    format!("{}{}   ", prefix, SYMBOL_DOWN)
                };
                for (i, router) in routers.iter().enumerate() {
                    print(f, &np, i == routers.len() - 1, router)?;
                }
            }
            Ok(())
        }
        print(f, "", true, self)
    }
}
