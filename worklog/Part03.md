# Main
The base function of this: 
```rust
#[handler]
async fn hello_world() -> &'static str {
    "Hello world"
}
```
Here are about How to build this marco `handler` : 

## `Handler` (src/handler.rs)

`Handler` is used for handle [`Request`].

* `Handler` can be used as middleware to handle [`Request`].

```rust
use salvo_core::prelude::*;
#[handler]
async fn middleware() { }
#[tokio::main]
async fn main() {
    Router::new().hoop(middleware);
}
```

* `Handler` can be used as endpoint to handle [`Request`].

```rust
# use salvo_core::prelude::*;
#[handler]
async fn middleware() { }
#[tokio::main]
async fn main() {
    Router::new().handle(middleware);
}
```

* `Handler` trait : 

```rust
#[async_trait]
pub trait Handler: Send + Sync + 'static {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }
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

`empty_handler`

This is a empty implement for `Handler` :

`empty_handler` does nothing except set [`Response`]'s status as [`StatusCode::OK`], it just marker a router exits.
```rust
#[allow(non_camel_case_types)]
pub struct empty_handler;
#[async_trait]
impl Handler for empty_handler {
    async fn handle(&self, _req: &mut Request, _depot: &mut Depot, res: &mut Response, _ctrl: &mut FlowCtrl) {
        res.set_status_code(StatusCode::OK);
    }
}
```

`Skipper` is used in many middlewares.
* check the request arrival wether should be skipped.
* implement for all type `Fn(&mut Request, &Depot) -> bool`, using self function
```rust
pub trait Skipper: Send + Sync + 'static {
    /// Check if the request should be skipped.
    fn skipped(&self, req: &mut Request, depot: &Depot) -> bool;
}
impl<F> Skipper for F
where
    F: Fn(&mut Request, &Depot) -> bool + Send + Sync + 'static,
{
    fn skipped(&self, req: &mut Request, depot: &Depot) -> bool {
        (self)(req, depot)
    }
}
```

`none_skipper` will skipper nothing.

It can be used as default `Skipper` in middleware.
```rust
pub fn none_skipper(_req: &mut Request, _depot: &Depot) -> bool {
    false
}
```

`Handler` and `Skipper` trait are implemented by `tuple`

```rust
macro_rules! handler_tuple_impls {
    (
        $(
            $Tuple:tt {
                $(($idx:tt) -> $T:ident,)+
            }
        )+
    ) => {
        $(
            #[async_trait::async_trait]
            impl<$($T,)+> Handler for ($($T,)+) where $($T: Handler,)+
            {
                async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl)
                {
                    $(
                        if !res.is_stamped() {
                            self.$idx.handler(req, depot, res, ctrl).await;
                        }
                    )+
                }
            }
        )+
    };
}
macro_rules! skipped_tuple_impls {
    (
        $(
            $Tuple:tt {
                $(($idx:tt) -> $T:ident,)+
            }
        )+
    ) => {
        $(
            impl<$($T,)+> Skipper for ($($T,)+) where $($T: Skipper,)+
            {
                fn skipped(&self, req: &mut Request, depot: &Depot) -> bool {
                    $(
                        if self.$idx.skipped(req, depot) {
                            return true;
                        }
                    )+
                    false
                }
            }
        )+
    };
}

macro_rules! __for_each_tuple {
    ($callback:ident) => {
        $callback! {
            1 {
                (0) -> A,
            }
            2 {
                (0) -> A,
                (1) -> B,
            }
            ... ...
        }
    };
}

__for_each_tuple!(handler_tuple_impls);
__for_each_tuple!(skipper_tuple_impls);
```

### `FlowCtrl` (src/routing/mod.rs)
`FlowCtrl` is used to control the flow of execute handlers.

* When a request is coming, [`Router`] will detect it and get the matched one.
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