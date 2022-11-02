# Main

Router part

## Router (src/routing/router.rs)

`Router` struct is used for route request to different handlers.

You can wite routers in flat way, like this:

```rust
#[tokio::main]
async fn main() {
    Router::with_path("writers").get(list_writers).post(create_writer);
    Router::with_path("writers/<id>").get(show_writer).patch(edit_writer).delete(delete_writer);
    Router::with_path("writers/<id>/articles").get(list_writer_articles);
}
```
You can write router like a tree, this is also the recommended way:
```rust
async fn main() {
    Router::with_path("writers")
        .get(list_writers)
        .post(create_writer)
        .push(
            Router::with_path("<id>")
                .get(show_writer)
                .patch(edit_writer)
                .delete(delete_writer)
                .push(Router::with_path("articles").get(list_writer_articles)),
    );
}
```
* This form of definition can make the definition of router clear and simple for complex projects.

Data struct : 
```rust
pub struct Router {
    pub(crate) routers: Vec<Router>,
    pub(crate) filters: Vec<Box<dyn Filter>>,
    pub(crate) hoops: Vec<Arc<dyn Handler>>,
    pub(crate) handler: Option<Arc<dyn Handler>>,
}
```
* impl `fmt::Debug` trait to debug type output.

Router detect result
```rust
pub struct DetectMatched {
    pub hoops: Vec<Arc<dyn Handler>>,
    pub handler: Arc<dyn Handler>,
}
```

__Functions__ : 

Detect current router is matched for current request.
```rust
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
    }
    if let Some(handler) = self.handler.clone() {
        if path_state.ended() {
            return Some(DetectMatched {
                hoops: self.hoops.clone(),
                handler: handler.clone(),
            });
        }
    }
    None
}
```

Router append
```rust
/// Push a router as child of current router.
pub fn push(mut self, router: Router) -> Self {
    self.routers.push(router);
    self
}
/// Append all routers in a Vec as children of current router.
pub fn append(mut self, others: Vec<Router>) -> Self {
    let mut others = others;
    self.routers.append(others);
    self
}
```

Router middleware additions
```rust
/// Add a handler as middleware, 
/// it will run the handler in current router or it's descendants
/// handle the request.
pub fn with_hoop<H: Handler>(handler: H) -> Self {
    Router::new().hoop(handler)
}
/// Add a handler as middleware, 
/// it will run the handler in current router or it's descendants
/// handle the request.
pub fn hoop<H: Handler>(mut self, handler: H) -> Self {
    self.hoops.push(Arc::new(handler));
    self
}
```

Router add path match
```rust
    /// Create a new router and set path filter.
    ///
    /// # Panics
    ///
    /// Panics if path value is not in correct format.
    pub fn with_path(path: impl Into<String>) -> Self {
        Router::with_filter(PathFilter::new(path))
    }
    /// Create a new path filter for current router.
    ///
    /// # Panics
    ///
    /// Panics if path value is not in correct format.
    pub fn path(self, path: impl Into<String>) -> Self {
        self.filter(PathFilter::new(path))
    }
```

Router add filter(s)
`FnFilter`
```rust
#[derive(Copy, Clone)]
pub struct FnFilter<F>(pub F);
```
* `F: Fn(&mut Request, &mut PathState) -> bool`
* implement `Filter` trait and `fmt::Debug` trait

```rust
/// Create a new router and set filter.
pub fn with_filter(filter: impl Filter + Sized) -> Self {
    Router::new().filter(filter)
}
/// Add a filter for current router.
pub fn filter(mut self, filter: impl Filter + Sized) -> Self {
    self.filters.push(Box::new(filter));
    self
}
/// Create a new router and set filter_fn.
pub fn with_filter_fn<T>(func: T) -> Self
where
    T: Fn(&mut Request, &mut PathState) -> bool + Send + Sync + 'static,
{
    Router::with_filter(FnFilter(func))
}
/// Create a new FnFilter from Fn.
pub fn filter_fn<T>(self, func: T) -> Self
where
    T: Fn(&mut Request, &mut PathState) -> bool + Send + Sync + 'static,
{
    self.filter(FnFilter(func))
}
```

Sets current router's handler.
```rust
pub fn handle<H: Handler>(mut self, handler: H) -> Self {
    self.handler = Some(Arc::new(handler));
    self
}
```

When you want write router chain, this function will be useful,

You can write your custom logic in FnOnce.
```rust
pub fn then<F>(self, func: F) -> Self
where
    F: FnOnce(Self) -> Self,
{
    func(self)
}
```

Add a `SchemeFilter` to current router.

`SchemeFilter` : super::filter::HostFilter
```rust
pub fn scheme(self, scheme: Scheme, default: bool) -> Self {
    self.filter(filter::scheme(scheme, default))
}
```

Add a `HostFilter` to current router.

`HostFilter`: super::filter::HostFilter
```rust
pub fn host(self, host: impl Into<String>, default: bool) -> Self {
    self.filter(filter::host(host, default))
}
```

Add a `PortFilter` to current router.

`PortFilter`: super::filter::PortFilter
```rust
pub fn port(self, port: u16, default: bool) -> Self {
    self.filter(filter::port(port, default))
}
```

Method Handler
```rust
/// Create a new child router with [`MethodFilter`] to filter get method and set this child router's handler.
///
/// [`MethodFilter`]: super::filter::MethodFilter
pub fn get<H: Handler>(self, handler: H) -> Self {
    self.push(Router::with_filter(filter::get()).handle(handler))
}

/// Create a new child router with [`MethodFilter`] to filter post method and set this child router's handler.
///
/// [`MethodFilter`]: super::filter::MethodFilter
pub fn post<H: Handler>(self, handler: H) -> Self {
    self.push(Router::with_filter(filter::post()).handle(handler))
}

/// Create a new child router with [`MethodFilter`] to filter put method and set this child router's handler.
///
/// [`MethodFilter`]: super::filter::MethodFilter
pub fn put<H: Handler>(self, handler: H) -> Self {
    self.push(Router::with_filter(filter::put()).handle(handler))
}

/// Create a new child router with [`MethodFilter`] to filter delete method and set this child router's handler.
///
/// [`MethodFilter`]: super::filter::MethodFilter
pub fn delete<H: Handler>(self, handler: H) -> Self {
    self.push(Router::with_filter(filter::delete()).handle(handler))
}

/// Create a new child router with [`MethodFilter`] to filter patch method and set this child router's handler.
///
/// [`MethodFilter`]: super::filter::MethodFilter
pub fn patch<H: Handler>(self, handler: H) -> Self {
    self.push(Router::with_filter(filter::patch()).handle(handler))
}

/// Create a new child router with [`MethodFilter`] to filter head method and set this child router's handler.
///
/// [`MethodFilter`]: super::filter::MethodFilter
pub fn head<H: Handler>(self, handler: H) -> Self {
    self.push(Router::with_filter(filter::head()).handle(handler))
}

/// Create a new child router with [`MethodFilter`] to filter options method and set this child router's handler.
///
/// [`MethodFilter`]: super::filter::MethodFilter
pub fn options<H: Handler>(self, handler: H) -> Self {
    self.push(Router::with_filter(filter::options()).handle(handler))
}
```