# Main

cfg_features : 

`acme` : 
* Automatically obtain a browser-trusted certificate, without any human intervention.
* about the let's encrypt : https://letsencrypt.org/how-it-works/

`native_tls` :
* Tls server

`rustls` : 
* Tls library written in rust.

`openssl` : 
* OpenSSL cryptography library.

`unix` : 
* For unix environments.

```rust
macro_rules! cfg_features {
    (
        #![$meta:meta]
        $($item:item)*
    ) => {
        $(
            #[cfg($meta)]
            #[cfg_attr(docsrs, doc(cfg($meta)))]
            #item
        )*
    }
}
```

## Acme (src/listener/acme)
ACME supports.

Reference: <https://datatracker.ietf.org/doc/html/rfc8555>
Reference: <https://datatracker.ietf.org/doc/html/rfc8737>

* HTTP-01
```rust
use salvo_core::listener::{AcmeListener, TcpListener};
use salvo_core::prelude::*;

#[handler]
async fn hello_world() -> &'static str {
    "Hello World"
}

#[tokio::main]
async fn main() {
    let mut router = Router::new().get(hello_world);
    let listener = AcmeListener::builder()
        // .directory("letsencrypt", salvo::listener::acme::LETS_ENCRYPT_STAGING)
        .cache_path("acme/letsencrypt")
        .add_domain("acme-http01.salvo.rs")
        .http01_challege(&mut router)
        .bind("0.0.0.0:443")
        .await;
    tracing::info!("Listening on https://0.0.0.0:443");
    Server::new(listener.join(TcpListener::bind("0.0.0.0:80")))
        .serve(router)
        .await;
}
```

* TLS ALPN-01
```rust
use salvo_core::listener::AcmeListener;
use salvo_core::prelude::*;

#[handler]
async fn hello_world() -> &'static str {
    "Hello World"
}

#[tokio::main]
async fn main() {
    let router = Router::new().get(hello_world);
    let listener = AcmeListener::builder()
        // .directory("letsencrypt", salvo::listener::acme::LETS_ENCRYPT_STAGING)
        .cache_path("acme/letsencrypt")
        .add_domain("acme-tls-alpn01.salvo.rs")
        .bind("0.0.0.0:443")
        .await;
    tracing::info!("Listening on https://0.0.0.0:443");
    Server::new(listener).serve(router).await;
}
```


+ [ ] Learn Acme Module and (Let's encrypt).
+ [ ] Others can reference to caddy.

---
## Native-Tls (src/listener/native_tls.rs)

---
## OpenSSl (src/listener/openssl.rs)

---
## Rustls (src/listener/rustls.rs)

---

## Mod (src/listener/mod.rs)

__use modules__ :

* `fastrand` : A simple and fast random number generator.

### Listener 
```rust
pub trait Listener: Accept {
    fn join<T>(self, other: T) -> JoinedListener<Self, T>
    where
        Self: Sized,
    {
        JoinedListener::new(self, other)
    }
}
```
`hyper::server::Accept` : 
Asynchronously accept incoming connections.


### JoinedStream
```rust
pub enum JoinedStream<A, B> {
    A(A),
    B(B),
}
```
impl `tokio::io::AsyncRead` trait 
* `fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>>`

impl `tokio::io::AsyncWrite` trait
* `fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>>`
* `fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>>`
* `fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>>`

impl `crate::transport::Transport` trait
* `fn remote_addr(&self) -> Option<RemoteAddr>`

### JoinedListener
```rust
pub struct JoinedListener<A, B> {
    a: A,
    b: B,
}
```

Limit : 
```rust
A: Accept + Send + Unpin + 'static,
B: Accept + Send + Unpin + 'static,
A::Conn: Transport,
B::Conn: Transport,
```

impl `Listener` trait

impl `hyper::server::Accept` trait 

In poll state-control-machine :
```rust
randomly control A or B Conn
match Pin::new(&mut pin.a).poll_accept(cx) {
             Poll::Ready(Some(result)) => Poll::Ready(Some(
                 result
                     .map(JoinedStream::A)
                     .map_err(|_| IoError::from(ErrorKind::Other)),
             )),
             Poll::Ready(None) => Poll::Ready(None),
             Poll::Pending => match Pin::new(&mut pin.b).poll_accept(cx) {
                 Poll::Ready(Some(result)) => Poll::Ready(Some(
                     result
                         .map(JoinedStream::B)
                         .map_err(|_| IoError::from(ErrorKind::Other)),
                 )),
                 Poll::Ready(None) => Poll::Ready(None),
                 Poll::Pending => Poll::Pending,
             },
         }
```

### TcpListener
```rust
pub struct TcpListener {
    incoming: AddrIncoming,
}
```

__Functions__ :

Get `AddrIncoming` of this listener.

`fn incoming(&self) -> &AddrIncoming`

Get the local address of this listener

`fn local_addr(&self) -> std::net::SocketAddr`

`fn bind(incoming: impl IntoAddrIncoming) -> Self`

`fn try_bind(incoming: impl IntoAddrIncoming) -> Result<Self, hyper::Error>`

__Traits__ :
* `Listener`
* `Accept`

### IntoAddrIncoming
```rust
pub trait IntoAddrIncoming {
    /// Convert into AddrIncoming
    fn into_incoming(self) -> AddrIncoming;
}
```

`std::net::SocketAddr` impl `IntoAddrIncoming` trait.
```rust
fn into_incoming(self) -> AddrIncoming {
    let mut incoming = AddrIncoming::bind(&self).unwrap();
    incoming.set_nodelay(true);
    incoming
}
```

`AddrIncoming` impl `IntoAddrIncoming` trait

`&<std::net::ToSocketAddrs + ?Sized>` impl `IntoAddrIncoming` trait

`(I: Into<std::net::ip::IpAddr>, u16)` impl `IntoAddrIncoming` trait
