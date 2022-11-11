mod opts;
mod others;
mod path;

use std::fmt::{self, Formatter};

use self::opts::*;
use crate::http::uri::Scheme;
use crate::http::{Method, Request};
use crate::routing::PathState;

pub use others::*;
pub use path::*;

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
    fn and_then<F>(self, fun: F) -> AndThen<Self, F>
    where
        Self: Sized,
        F: Fn(&mut Request, &mut PathState) -> bool + Send + Sync + 'static,
    {
        AndThen {
            filter: self,
            callback: fun,
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
pub fn path(path: impl Into<String>) -> PathFilter {
    PathFilter::new(path)
}
pub fn get() -> MethodFilter {
    MethodFilter(Method::GET)
}
pub fn head() -> MethodFilter {
    MethodFilter(Method::HEAD)
}
pub fn options() -> MethodFilter {
    MethodFilter(Method::OPTIONS)
}
pub fn post() -> MethodFilter {
    MethodFilter(Method::POST)
}
pub fn patch() -> MethodFilter {
    MethodFilter(Method::PATCH)
}
pub fn put() -> MethodFilter {
    MethodFilter(Method::PUT)
}
pub fn delete() -> MethodFilter {
    MethodFilter(Method::DELETE)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_methods() {
        assert!(get() == MethodFilter(Method::GET));
        assert!(head() == MethodFilter(Method::HEAD));
        assert!(options() == MethodFilter(Method::OPTIONS));
        assert!(post() == MethodFilter(Method::POST));
        assert!(patch() == MethodFilter(Method::PATCH));
        assert!(put() == MethodFilter(Method::PUT));
        assert!(delete() == MethodFilter(Method::DELETE));
    }

    #[test]
    fn test_opts() {
        fn has_one(_req: &mut Request, path: &mut PathState) -> bool {
            path.parts.contains(&"one".into())
        }
        fn has_two(_req: &mut Request, path: &mut PathState) -> bool {
            path.parts.contains(&"two".into())
        }

        let one_filter = FnFilter(has_one);
        let two_filter = FnFilter(has_two);

        let mut req = Request::default();
        let mut path_state = PathState::new("http://localhost/one");
        assert!(one_filter.filter(&mut req, &mut path_state));
        assert!(!two_filter.filter(&mut req, &mut path_state));
        assert!(one_filter
            .or_else(has_two)
            .filter(&mut req, &mut path_state));
        assert!(one_filter.or(two_filter).filter(&mut req, &mut path_state));
        assert!(!one_filter
            .and_then(has_two)
            .filter(&mut req, &mut path_state));
        assert!(!one_filter.and(two_filter).filter(&mut req, &mut path_state));

        let mut path_state = PathState::new("http://localhost/one/two");
        assert!(one_filter.filter(&mut req, &mut path_state));
        assert!(two_filter.filter(&mut req, &mut path_state));
        assert!(one_filter
            .or_else(has_two)
            .filter(&mut req, &mut path_state));
        assert!(one_filter.or(two_filter).filter(&mut req, &mut path_state));
        assert!(one_filter
            .and_then(has_two)
            .filter(&mut req, &mut path_state));
        assert!(one_filter.and(two_filter).filter(&mut req, &mut path_state));

        let mut path_state = PathState::new("http://localhost/two");
        assert!(!one_filter.filter(&mut req, &mut path_state));
        assert!(two_filter.filter(&mut req, &mut path_state));
        assert!(one_filter
            .or_else(has_two)
            .filter(&mut req, &mut path_state));
        assert!(one_filter.or(two_filter).filter(&mut req, &mut path_state));
        assert!(!one_filter
            .and_then(has_two)
            .filter(&mut req, &mut path_state));
        assert!(!one_filter.and(two_filter).filter(&mut req, &mut path_state));
    }
}
