# Main

Finish Test module with client -- `Request` Part.

## Request Builder

__use crate__ :

* `url` : rust-url is an implementation of the URL Standard for the Rust programming language.
* `base64` 
* `serde_urlencoded` : x-www-form-urlencoded meets Serde.

### `RequestBuilder`  (src/test/request/builder.rs)
It's the main way of building requests.

You can create a `RequestBuilder` using the `new` or `try_new` method, but the recommended way
or use one of the simpler constructors available in the crate root or on the `Session` struct,
such as `get`, `post`, etc.
```rust
#[derive(Debug)]
pub struct RequestBuilder {
    url: Url,
    method: Method,
    headers: HeaderMap,
    // params: HashMap<String, String>,
    body: Body,
}
```

__Functions__ : 


`fn new<U>(url: U, method: Method) -> Self where U: AsRef<str>`

Create a new `RequestBuilder` with the base URL and the given method.

__Panics__ : Panics if the base url is invalid or if the method is CONNECT.

---
`fn query<K, V>(mut self, key: K, value: V) -> Self`
* `K: AsRef<str>`
* `V: ToString`

Associate a query string parameter to the given value.

The same key can be used multiple times.

---

`fn queries<P, K, V>(mut self, pairs: P) -> Self`
* `P: IntoIterator`
* `P::Item: Borrow<(K, V)>`
* `K: AsRef<str>`
* `V: ToString`

Associated a list of pairs to query parameters.

The same key can be used multiple times.

```rust
TestClient::get("http://foo.bar").queries(&[("p1", "v1"), ("p2", "v2")]);
```

---
`fn param<K, V>(mut self, key: K, value: V) -> Self`
* `K: AsRef<str>`
* `V: ToString`

Associate a url param to the given value.

---
`fn params<P, K, V>(mut self, pairs: P) -> Self`
* `P: IntoIterator`
* `P::Item: Borrow<(K, V)>`
* `K: AsRef<str>`
* `V: ToString`

Associated a list of url params.

---
`fn basic_auth(self, username: impl std::fmt::Display, password: Option<impl std::fmt::Display>) -> Self`

Enable HTTP basic authentication.

---
`fn bearer_auth(self, token: impl Into<String>) -> Self`

Enable HTTP bearer authentication.

---
`fn body(mut self, body: impl Into<Body>) -> Self`

Sets the body of this request.

---
`fn text(mut self, body: impl Into<String>) -> Self`

Sets the body of this request to be text.

If the `Content-Type` header is unset, it will be set to `text/plain` and the charset to UTF-8.

---
`fn bytes(mut self, body: Vec<u8>) -> Self`

Sets the body of this request to be bytes.

If the `Content-Type` header is unset, it will be set to `application/octet-stream`.

---
`fn json<T: serde::Serialize>(mut self, value: &T) -> Self`

Sets the body of this request to be the JSON representation of the given object.

If the `Content-Type` header is unset, it will be set to `application/json` and the charset to UTF-8.

---
`fn raw_json(mut self, value: impl Into<String>) -> Self`

Sets the body of this request to be the JSON representation of the given string.

If the `Content-Type` header is unset, it will be set to `application/json` and the charset to UTF-8.

---
`fn form<T: serde::Serialize>(mut self, value: &T) -> Self`

Sets the body of this request to be the URL-encoded representation of the given object.

If the `Content-Type` header is unset, it will be set to `application/x-www-form-urlencoded`.

---
`fn raw_form(mut self, value: impl Into<String>) -> Self`

Sets the body of this request to be the URL-encoded representation of the given string.

If the `Content-Type` header is unset, it will be set to `application/x-www-form-urlencoded`.

---
`fn add_header<N, V>(mut self, name: N, value: V, overwrite: bool) -> Self`
* `N: IntoHeaderName`
* `V: TryInto<HeaderValue>`

Modify a header for this response.
* When `overwrite` is set to `true`, If the header is already present, the value will be replaced.
* When `overwrite` is set to `false`, The new header is always appended to the request, even if the header already exists.

---
`fn build(self) -> Request`

Build final request.

---
`fn build_hyper(self) -> hyper::Request<Body>`

Build hyper request.

---
`async fn send(self, target: impl SendTarget) -> Response`

Send request to target, such as [`Router`], [`Service`], [`Handler`].

### SendTarget
```rust
#[async_trait]
pub trait SendTarget {
    async fn call(self, req: Request) -> Response;
}
```

impl `SendTarget` for 
`&crate::service::Service` , 
`crate::routing::Router` and `Arc<Router>`

impl `SendTarget` for `Arc<T>`
* `T: crate::handler::Handler + Send`
```rust
async fn call(self, req: Request) -> Response {
    let mut req = req;
    let mut depot = Depot::new();
    let mut res = Response::with_cookies(req.cookies.clone());
    let mut ctrl = FlowCtrl::new(vec![self.clone()]);
    self.handle(&mut req, &mut depot, &mut res, &mut ctrl).await;
    res
}
```

impl `SendTarget` for `T`
* `T: crate::handler::Handler + Send`
