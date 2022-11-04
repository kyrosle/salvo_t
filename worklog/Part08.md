# Main

Build the Server.

## Server

```rust
pub struct Server<L> {
    listener: L,
}
```
Limit: 

* `L: Listener`
* `L::Conn: Transport + Send + Unpin + 'static`
* `L::Error: Into<Box<(dyn StdError + Send + Sync + 'static)>>`

using `hyper::Server` to start a server.

__Functions__ :


Create new `Server` with `Listener`.

```rust
use salvo_core::prelude::*;
  
#[tokio::main]
async fn main() {
    Server::new(TcpListener::bind("127.0.0.1:7878"));
}
```

`fn new(listener: L) -> Self`

`async fn serve<S>(self, service: S) where S: Into<Service>`

`async fn try_serve<S>(self, service: S) -> Result<(), hyper::Error> where S: Into<Service>`

---
Serve with graceful shutdown signal.

```rust
use tokio::sync::oneshot;

use salvo_core::prelude::*;

#[handler]
async fn hello_world(res: &mut Response) {
    res.render("Hello World!");
}

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel();
    let router = Router::new().get(hello_world);
    let server = Server::new(
        TcpListener::bind("127.0.0.1:7878"))
            .serve_with_graceful_shutdown(
                router,
                async {
                        rx.await.ok();
                }
    );

    // Spawn the server into a runtime
    tokio::task::spawn(server);

    // Later, start the shutdown...
    let _ = tx.send(());
}
```

`async fn serve_with_graceful_shutdown<S, G>(self, addr: S, signal: G) `
* `where S: Into<Service>`
* `G: Future<Output = ()> + Send + 'static`

`async fn try_serve_with_graceful_shutdown<S, G>(self, service: S, signal: G) `
* `where S: Into<Service>`
* `G: Future<Output = ()> + Send + 'static`