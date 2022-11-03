# Main

The following : `Catcher` and `Transport` then `Service` 

## `Catcher` (src/catcher.rs)

### `Catcher`

`Catcher` trait and it's default implement `CatcherImpl`.

A web application can specify several different Catchers to handle errors.

They can be set via the ```with_catchers``` function of ```Server```:

```rust
# use salvo_core::prelude::*;
# use salvo_core::Catcher;

struct Handle404;
impl Catcher for Handle404 {
    fn catch(&self, _req: &Request, _depot: &Depot, res: &mut Response) -> bool {
        if let Some(StatusCode::NOT_FOUND) = res.status_code() {
            res.render("Custom 404 Error Page");
            true
        } else {
            false
        }
    }
}

#[tokio::main]
async fn main() {
    let catchers: Vec<Box<dyn Catcher>> = vec![Box::new(Handle404)];
    Service::new(Router::new()).with_catchers(catchers);
}
```

When there is an error in the website request result, first try to set the error page
through the `Catcher` set by the user. If the `Catcher` catches the error,
it will return `true`.

If your custom catchers does not capture this error, then the system uses the
default `CatcherImpl` to capture processing errors and send the default error page.

__Trait__ :

Catch http response error.
* If the current catcher caught the error, it will returns true.
* If current catcher is not interested in current error, it will returns false.
 Salvo will try to use next catcher to catch this error.
* If all custom catchers can not catch this error, `CatcherImpl` will be used
 to catch it.
```rust
pub trait Catcher: Send + Sync + 'static {
    fn catch(&self, req: &Request, depot: &Depot, res: &mut Response) -> bool;
}
```

__Functions__ :
```rust
fn status_error_html( code: StatusCode, name: &str, summary: Option<&str>, detail: Option<&str>) -> String
fn status_error_json(code: StatusCode, name: &str, summary: Option<&str>, detail: Option<&str>) -> String 
fn status_error_plain(code: StatusCode, name: &str, summary: Option<&str>, detail: Option<&str>) -> String 
fn status_error_xml(code: StatusCode, name: &str, summary: Option<&str>, detail: Option<&str>) -> String 
fn status_error_bytes(err: &StatusError, prefer_format: &Mime) -> (Mime, Vec<u8>) 
```

### `CatcherImpl`

Default implementation of `Catcher`.

* If http status is error, and user is not set custom catcher to catch them,
`CatcherImpl` will catch them.

* `CatcherImpl` supports sending error pages in `XML`, `JSON`, `HTML`, `Text` formats.
```rust
pub struct CatcherImpl;
```
* impl `Catcher` trait

assist function :
(src/http/mod.rs)
```rust
fn guess_accept_mime(req: &Request, default_type: Option<Mime>) -> Mime
```

Fn `Catcher::catch`
```rust
    fn catch(&self, req: &mut Request, depot: &mut Depot, res: &mut Response) -> bool {
        let status = res.status_code().unwrap_or(StatusCode::NOT_FOUND);
        if !status.is_server_error() && !status.is_client_error() {
            return false;
        }
        let format = guess_accept_mine(req, None);
        let (format, data) = if res.status_error.is_some() {
            status_error_bytes(res.status_error.as_ref().unwrap(), &format)
        } else {
            status_error_bytes(&StatusError::from_code(status).unwrap(), &format)
        };
        res.headers_mut()
            .insert(header::CONTENT_TYPE, format.to_string().parse().unwrap());
        res.write_body(data).ok();
        true
    }
```
## Transport (src/transport.rs)

* `tokio::io::AsyncRead`
* `tokio::io::AsyncWrite`
* `hyper::server::conn::AddrStream`

```rust
pub trait Transport: AsyncRead + AsyncWrite {
    fn remote_addr(&self) -> Option<SocketAddr>;
}

impl Transport for AddrStream {
    fn remote_addr(&self) -> Option<SocketAddr> {
        Some(self.remote_addr().into())
    }
}
```

## Service (src/service.rs)

Service http request.
```rust
pub struct Service {
    pub(crate) router: Arc<Router>,
    pub(crate) catchers: Arc<Vec<Box<dyn Catcher>>>,
    pub(crate) allowed_media_types: Arc<Vec<Mime>>,
}
```

__Functions__ : 

Create a new Service with a `Router`.

`fn new<T>(router: T) -> Service`
* where `T: Into<Arc<Router>>`


`fn with_catchers<T>(mut self, catchers: T) -> Self`
* where `T: Into<Arc<Vec<Box<dyn Catcher>>>>`

Get allowed media types list.

`fn with_allow_catchers<T>(mut self, allowed_media_types: T) -> Self`
* where `T: Into<Arc<Vec<Mime>>>`

using struct :
```rust
#[derive(Clone)]
pub struct HyperHandler {
    pub(crate) remote_addr: Option<SocketAddr>,
    pub(crate) router: Arc<Router>,
    pub(crate) catchers: Arc<Vec<Box<dyn Catcher>>>,
    pub(crate) allowed_media_types: Arc<Vec<Mime>>,
}
```
__Having function__ :
```rust
impl HyperHandler {
    pub fn handle(&self, mut req: Request) -> impl Future<Output = Response> {
        let catchers = self.catchers.clone();
        let allowed_media_types = self.allowed_media_types.clone();
        req.remote_addr = self.remote_addr.clone();
        let mut res = Response::with_cookies(req.cookies.clone());
        let mut depot = Depot::new();
        let mut path_state = PathState::new(req.uri().path());
        let router = self.router.clone();

        async move {
            if let Some(dm) = router.detect(&mut req, &mut path_state) {
                req.params = path_state.params;
                let mut ctrl = FlowCtrl::new([&dm.hoops[..], &[dm.handler]].concat());
                ctrl.call_next(&mut req, &mut depot, &mut res).await;
            } else {
                res.set_status_code(StatusCode::NOT_FOUND);
            }

            if res.status_code().is_none() {
                if res.body.is_none() {
                    res.set_status_code(StatusCode::NOT_FOUND);
                } else {
                    res.set_status_code(StatusCode::OK);
                }
            }

            let status = res.status_code().unwrap();
            let has_error = status.is_client_error() || status.is_server_error();
            if let Some(value) = res.headers().get(CONTENT_TYPE) {
                let mut is_allowed = false;
                if let Ok(value) = value.to_str() {
                    if allowed_media_types.is_empty() {
                        is_allowed = true;
                    } else {
                        let ctype: Result<Mime, _> = value.parse();
                        if let Ok(ctype) = ctype {
                            for mime in &*allowed_media_types {
                                if mime.type_() == ctype.type_()
                                    && mime.subtype() == ctype.subtype()
                                {
                                    is_allowed = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                if !is_allowed {
                    res.set_status_code(StatusCode::UNSUPPORTED_MEDIA_TYPE);
                }
            } else if res.body.is_none() && !has_error {
                tracing::warn!(
                    url =?req.uri(),
                    method = req.method().as_str(),
                    "Http response content type header not set"
                );
            }
            if res.body.is_none() && has_error {
                let mut catch = false;
                for catcher in catchers.iter() {
                    if catcher.catch(&req, &depot, &mut res) {
                        catch = true;
                        break;
                    }
                }
                if !catch {
                    CatcherImpl.catch(&req, &depot, &mut res);
                }
            }
            if let hyper::Method::HEAD = *req.method() {
                if !res.body.is_none() {
                    tracing::warn!("request with head method should not have body: https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/HEAD");
                }
            }
            res
        }
    }
}
```
`pub fn hyper_handler(&self, remote_addr: Option<SocketAddr>) -> HyperHandler `


| Handle `Request` and returns `Response`.

This function is useful for testing application.

```rust
use salvo_core::prelude::*;
use salvo_core::test::{ResponseExt, TestClient};

#[handler]
async fn hello_world() -> &'static str {
    "Hello World"
}
#[tokio::main]
async fn main() {
    let service: Service = Router::new().get(hello_world).into();
    let mut res = TestClient::get("http://127.0.0.1:7878").send(&service).await;
    assert_eq!(res.take_string().await.unwrap(), "Hello World");
}
```
`pub async fn handler(&self, request: impl Into<Request>) -> Response`
