use std::fmt::{self, Formatter};

pub mod opts;
pub mod others;
pub mod path;

use hyper::http::uri::Scheme;
pub use others::*;
pub use path::*;

use crate::http::request::Request;

use self::opts::*;

use super::PathState;

pub trait Filter: fmt::Debug + Send + Sync + 'static {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    fn and<F>(self, other: F) -> And<Self, F>
    where
        Self: Sized,
        F: Filter + Send + Sync,
    {
        And {
            first: self,
            second: other,
        }
    }
    fn or<F>(self, other: F) -> Or<Self, F>
    where
        Self: Sized,
        F: Filter + Send + Sync,
    {
        Or {
            first: self,
            second: other,
        }
    }
    fn and_then<F>(self, other: F) -> AndThen<Self, F>
    where
        Self: Sized,
        F: Fn(&mut Request, &mut PathState) -> bool + Send + Sync + 'static,
    {
        AndThen {
            filter: self,
            callback: other,
        }
    }
    fn or_else<F>(self, fun: F) -> OrElse<Self, F>
    where
        Self: Sized,
        F: Fn(&mut Request, &mut PathState) -> bool + Send + Sync + 'static,
    {
        OrElse {
            filter: self,
            callback: fun,
        }
    }

    fn filter(&self, req: &mut Request, path: &mut PathState) -> bool;
}

#[derive(Copy, Clone)]
#[allow(missing_debug_implementations)]
pub struct FnFilter<F>(pub F);

impl<F> Filter for FnFilter<F>
where
    F: Fn(&mut Request, &mut PathState) -> bool + Send + Sync + 'static,
{
    fn filter(&self, req: &mut Request, path: &mut PathState) -> bool {
        self.0(req, path)
    }
}
impl<F> fmt::Debug for FnFilter<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "fn:fn")
    }
}

pub fn scheme(scheme: Scheme, default: bool) -> SchemeFilter {
    SchemeFilter(scheme, default)
}
pub fn host(host: impl Into<String>, default: bool) -> HostFilter {
    HostFilter(host.into(), default)
}
pub fn port(port: u16, default: bool) -> PortFilter {
    PortFilter(port, default)
}
pub fn path(path: String, default: bool) -> PathFilter {
    PathFilter(path.into(), default)
}
