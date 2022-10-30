use crate::{http::request::Request, routing::PathState};

use super::Filter;

#[derive(Debug, Clone)]
pub struct Or<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

#[derive(Copy, Clone)]
pub struct OrElse<T, F> {
    pub(super) filter: T,
    pub(super) callback: F,
}

#[derive(Debug, Clone)]
pub struct And<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

#[derive(Copy, Clone)]
pub struct AndThen<T, F> {
    pub(super) filter: T,
    pub(super) callback: F,
}

impl<T, U> Filter for Or<T, U>
where
    T: Filter + Send,
    U: Filter + Send,
{
    fn filter(&self, req: &mut Request, state: &mut PathState) -> bool {
        if self.first.filter(req, state) {
            true
        } else {
            self.second.filter(req, state)
        }
    }
}

impl<T, F> Filter for OrElse<T, F>
where
    T: Filter,
    F: Fn(&mut Request, &mut PathState) -> bool + Send + Sync + 'static,
{
    fn filter(&self, req: &mut Request, state: &mut PathState) -> bool {
        if self.filter(req, state) {
            true
        } else {
            (self.callback)(req, state)
        }
    }
}
