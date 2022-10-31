# Main

Router part

## Router (src/routing/router.rs)

`Router` struct is used for route request to different handlers.

You can wite routers in flat way, like this:

```rust
use salvo_core::prelude::*;

#[handler]
async fn create_writer(res: &mut Response) { }
#[handler]
async fn show_writer(res: &mut Response) { }
#[handler]
async fn list_writers(res: &mut Response) { }
#[handler]
async fn edit_writer(res: &mut Response) { }
#[handler]
async fn delete_writer(res: &mut Response) { }
#[handler]
async fn list_writer_articles(res: &mut Response) { }

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

```rust
/// Push a router as child of current router.
pub fn push(mut self, router: Router) -> Self {
    self.routers.push(router);
    self
}
/// Append all routers in a Vec as children of current router.
pub fn append(mut self, others: Vec<Router>) -> Self {
    let mut others = others;
    self.routers.append(&mut others);
    self
}
```

```rust
/// Add a handler as middleware, it will run the handler in current router or it's descendants
/// handle the request.
pub fn with_hoop<H: Handler>(handler: H) -> Self {
    Router::new().hoop(handler)
}

/// Add a handler as middleware, it will run the handler in current router or it's descendants
/// handle the request.
pub fn hoop<H: Handler>(mut self, handler: H) -> Self {
    self.hoops.push(Arc::new(handler));
    self
}
```

```rust
    /// Create a new router and set path filter.
    ///
    /// # Panics
    ///
    /// Panics if path value is not in correct format.
    #[inline]
    pub fn with_path(path: impl Into<String>) -> Self {
        Router::with_filter(PathFilter::new(path))
    }

    /// Create a new path filter for current router.
    ///
    /// # Panics
    ///
    /// Panics if path value is not in correct format.
    #[inline]
    pub fn path(self, path: impl Into<String>) -> Self {
        self.filter(PathFilter::new(path))
    }
```

```rust
/// Create a new router and set filter.
#[inline]
pub fn with_filter(filter: impl Filter + Sized) -> Self {
    Router::new().filter(filter)
}
/// Add a filter for current router.
#[inline]
pub fn filter(mut self, filter: impl Filter + Sized) -> Self {
    self.filters.push(Box::new(filter));
    self
}
```