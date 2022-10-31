use hyper::{http::uri::Scheme, Method};
use std::fmt;

use crate::{http::request::Request, routing::PathState};

use super::Filter;

#[derive(Clone, PartialEq, Eq)]
pub struct MethodFilter(pub Method);

impl Filter for MethodFilter {
    fn filter(&self, req: &mut Request, path: &mut PathState) -> bool {
        req.method() == self.0
    }
}
impl fmt::Debug for MethodFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "method: {:?}", self.0)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SchemeFilter(pub Scheme, pub bool);

impl Filter for SchemeFilter {
    fn filter(&self, req: &mut Request, path: &mut PathState) -> bool {
        req.uri().scheme().map(|s| s == &self.0).unwrap_or(self.1)
    }
}
impl fmt::Debug for SchemeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "scheme: {:?}", self.0)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct HostFilter(pub String, pub bool);

impl Filter for HostFilter {
    fn filter(&self, req: &mut Request, path: &mut PathState) -> bool {
        req.uri().host().map(|h| h == self.0).unwrap_or(self.1)
    }
}
impl fmt::Debug for HostFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "host: {:?}", self.0)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct PortFilter(pub u16, pub bool);

impl Filter for PortFilter {
    fn filter(&self, req: &mut Request, path: &mut PathState) -> bool {
        req.uri().port_u16().map(|p| p == self.0).unwrap_or(self.1)
    }
}
impl fmt::Debug for PortFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "port: {:?}", self.0)
    }
}
