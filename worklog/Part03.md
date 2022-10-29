# Part03

The base function of this: 
```rust
#[handler]
async fn hello_world() -> &'static str {
    "Hello world"
}
```
For this marco `handler`

---
## `Handler` (src/handler.rs)

```rust
#[async_trait]
pub trait Handler: Send + Sync + 'static {
    #[doc(hidden)]
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }
    #[doc(hidden)]
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    /// Handle http request.
    #[must_use = "handle future must be used"]
    async fn handle(
        &self, 
        req: &mut Request, 
        depot: &mut Depot,
        res: &mut Response, 
        ctrl: &mut FlowCtrl
    );
}
```


### `FlowCtrl` (src/routing)
`FlowCtrl` is used to control the flow of execute handlers.

When a request is coming, [`Router`] will detect it and get the matched one.
And then salvo will collect all handlers (including added as middlewares) in a list.
All handlers in this list will executed one by one. Each handler can use `FlowCtrl` to control this
flow, let the flow call next handler or skip all rest handlers.

**NOTE**: When `Response`'s status code is set, and it's `is_success()` is returns false, all rest handlers
will skipped.

 [`Router`]: crate::routing::Router
```rust
pub struct FlowCtrl {
    is_ceased: bool,
    cursor: usize,
    pub(crate) handlers: Vec<Arc<dyn Handler>>,
}
```

Call next handler. If get next handler and executed, returns true, otherwise returns false.

If response's status code is error or is redirection, all reset handlers will skipped.
```rust
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
```