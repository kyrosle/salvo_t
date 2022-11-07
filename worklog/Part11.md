# Main

## Client
```rust
#[derive(Debug, Default)]
pub struct TestClient;
```

__Functions__ :

`get`, `post`, `put`, `delete`, `patch`, `head`, `options`, `trace`

## Response

More utils functions for response.


__use modules__ : 
* `bytes` : An efficient container for storing and operating on contiguous slices of memory. It is intended for use primarily in networking code, but could have applications elsewhere as well.

* `encoding_rs` :  Converting to and from UTF-16 is supported in addition to converting to and from UTF-8

* `async_compression` : Adaptors between compression crates and Rust’s modern asynchronous IO types.

```rust
#[async_trait]
pub trait ResponseExt {
    /// Take body as ```String``` from response.
    async fn take_string(&mut self) -> crate::Result<String>;
    /// Take body as deserialize it to type `T` instance.
    async fn take_json<T: DeserializeOwned>(&mut self) -> crate::Result<T>;
    /// Take body as ```String``` from response with charset.
    async fn take_string_with_charset(&mut self, charset: &str, compress: Option<&str>) -> crate::Result<String>;
    /// Take all body bytes. If body is none, it will creates and returns a new [`Bytes`].
    async fn take_bytes(&mut self) -> crate::Result<Bytes>;
}
```

impl `ResponseExt` for `crate::http::Response`
