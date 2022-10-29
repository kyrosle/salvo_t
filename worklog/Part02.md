# Main
Building base router function

## Depot (src/http/depot.rs)
data-struct-field:
```rust
#[derive(Default)]
pub struct Depot {
    map: HashMap<String, Box<dyn Any + Send + Sync>>,
}
```

---
## Request (src/http/request.rs)
use crate (only enum the module not the sub function or struct , enum etc.)

__using outer crate__ :

* `cookie`
open the features : percent-encode

* `hyper`
open the features : stream, server, http1, http2, tcp, client
http::header
http::method
http::version
http::{Extension, Uri}

* `mime`
Media Type

* `multimap`
wrapper around `std::collections::HashMap` 
but allow same key,
just like the value is a vector.

* `once_cell`
just like the `lazy_static`

* `serde` Serializer and Deserializer

* `serde_json` for its `RawValue`

* `form_urlencoded` : Convert a byte string in the `application/x-www-form-urlencoded` syntax into a `iterator` of `(name, value)` pairs (collected as `HashMap` such as).
- form_urlencoded

| pub           | pub(crate)        | pub(super)         | pub(self)          |
| ------------- | ----------------- | ------------------ | ------------------ |
| for every one | for current crate | for parent modules | for current module |

Data struct : 
```rust
pub struct Request {
    // request url
    uri: Uri,
    // request header
    headers: HeaderMap,
    // request body as a reader
    body: Option<Body>,
    extensions: Extensions,
    // request method
    method: Method,
    // accept: Option<Vec<Mime>>,
    pub(crate) queries: OnceCell<MultiMap<String, String>>,
    pub(crate) form_data: tokio::sync::OnceCell<FormData>,
    pub(crate) payload: tokio::sync::OnceCell<Vec<u8>>,

    version: Version,
    pub(crate) remote_addr: Option<SocketAddr>,
}
```

__functions__ :

Read field with `T(take)`, `&T(&)`, `&mut T(&mut)` types functions.

`from_data(&mut self) -> Result<&FormData, ParseError>` :
* read from `self.headers` get the `content_type` as `ctype`
and then match the `ctype` 
to construct the `FormData` with the `body` in `self.form_data`.

`file<'a>(&'a mut self, key: &'a str) -> Option<'a FilePart>` : 
* read file data from the `self.form_data`.

`payload(&mut self) -> Result<&Vec<u8>, ParseError>` : 
* read from body like `json` etc.

`extract_with_metadata<'de, T>(&'de mut self, metadata: &'de Metadata) -> Result<T, ParseError>` : 
* use `from_request(self, metadata)` to get from `self.form_data` and `self.payload()`. 

Parsing Self Value

`pub fn accept(&self) -> Vec<Mime>` 
* Get Accept

---
## Response (src/http/response.rs)

### `ResBody`
Response body type.

Data struct :
```rust
#[allow(clippy::type_complexity)]
#[non_exhaustive]
pub enum ResBody {
    /// None body.
    None,
    /// Once bytes body.
    Once(Bytes),
    /// Chunks body.
    Chunks(VecDeque<Bytes>),
    /// Stream body.
    Stream(Pin<Box<dyn Stream<Item = Result<Bytes, Box<dyn StdError + Send + Sync>>> + Send>>),
}
```
__Implements__ : 

`Stream` trait like a futures state machine.
```rust
impl Stream for ResBody {
    type Item = Result<Bytes, Box<dyn StdError + Send + Sync>>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.get_mut() {
            ResBody::None => Poll::Ready(None),
            ResBody::Once(bytes) => {
                if bytes.is_empty() {
                    Poll::Ready(None)
                } else {
                    let bytes = std::mem::replace(bytes, Bytes::new());
                    Poll::Ready(Some(Ok(bytes)))
                }
            }
            ResBody::Chunks(chunks) => Poll::Ready(chunks.pop_front().map(Ok)),
            ResBody::Stream(stream) => stream.as_mut().poll_next(cx),
        }
    }
}
```

### `Response`
Data struct:
Represents an HTTP response
```rust
pub struct Response {
    status_code: Option<StatusCode>,
    pub(crate) status_error: Option<StatusError>,
    headers: HeaderMap,
    version: Version,
    pub(crate) cookies: CookieJar,
    pub(crate) body: ResBody,
}
```

impl `From<hyper::Response<hyper::Body>>` trait

__use module__ :

`tokio_stream` : It can be thought of as an asynchronous version of the standard library’s Iterator trait.


If return `true`, it means this response is ready for write back and the reset handlers should be skipped.
```rust
pub fn is_stamped(&mut self) -> bool {
    if let Some(code) = self.status_code() {
        if code.is_client_error() || code.is_server_error() || code.is_redirection() {
            return true;
        }
    }
    false
}
```

`write_back` is used to put all the data added to `self`
back onto an `hyper::Response` so that it is sent back to the
client.  And `write_back` consumes the `Response`.
main functions: 
```rust
    pub fn write_cookies_to_headers(&mut self) {
        for cookie in self.cookies.delta() {
            if let Ok(hv) = cookie.encoded().to_string().parse() {
                self.headers.append(SET_COOKIE, hv);
            }
        }
        self.cookies = CookieJar::new();
    }
    pub(crate) async fn write_back(mut self, res: &mut hyper::Response<hyper::Body>) {
        self.write_cookies_to_headers();
        let Self {
            status_code,
            headers,
            body,
            ..
        } = self;
        *res.headers_mut() = headers;

        // Default to a 404 if no response code was set
        *res.status_mut() = status_code.unwrap_or(StatusCode::NOT_FOUND);
        
        match body {
            ResBody::None => {
                res.headers_mut().insert(CONTENT_LENGTH, HeaderValue::from_static("0"));
            }
            ResBody::Once(bytes) => {
                *res.body_mut() = hyper::Body::from(bytes);
            }
            ResBody::Chunks(chunks) => {
                *res.body_mut() = hyper::Body::wrap_stream(tokio_stream::iter(
                    chunks.into_iter().map(Result::<_, Box<dyn StdError + Sync + Send>>::Ok)
                ));
            }
            ResBody::Stream(stream) => {
                *res.body_mut() = hyper::Body::wrap_stream(stream);
            }
        }
    }
```

Render content.
```rust
pub fn render<P>(&mut self, piece: P)
where
    P: Piece,
{
    piece.render(self)
}
pub fn with_render<P>(&mut self, piece: P) -> &mut Self
where
    P: Piece,
{
    self.render(piece);
    self
}
```

Render content with status code.
```rust
pub fn stuff<P>(&mut self, code: StatusCode, piece: P)
where
    P: Piece,
{
    self.status_code = Some(code);
    piece.render(self)
}
pub fn with_stuff<P>(&mut self, code: StatusCode, piece: P) -> &mut Self
where
    P: Piece,
{
    self.stuff(code, piece);
    self
}
```

Write bytes data to body. If body is none, a new `ResBody` will created.
```rust
pub fn write_body(&mut self, data: impl Into<Bytes>) -> crate::Result<()> {
    match self.body_mut() {
        ResBody::None => {
            self.body = ResBody::Once(data.into());
        }
        ResBody::Once(ref bytes) => {
            let mut chunks = VecDeque::new();
            chunks.push_back(bytes.clone());
            chunks.push_back(data.into());
            self.body = ResBody::Chunks(chunks);
        }
        ResBody::Chunks(chunks) => {
            chunks.push_back(data.into());
        }
        ResBody::Stream(_) => {
            tracing::error!(
                "current body kind is `ResBody::Stream`, try to write byres to it "
            );
            return Err(Error::other(
                "current body kind is `ResBody::Stream`, try to write byres to it ",
            ));
        }
    }
    Ok(())
}
```

Write streaming data.
```rust
#[inline]
pub fn streaming<S, O, E>(&mut self, stream: S) -> crate::Result<()>
where
    S: Stream<Item = Result<O, E>> + Send + 'static,
    O: Into<Bytes> + 'static,
    E: Into<Box<dyn StdError + Send + Sync>> + 'static,
{
    match &self.body {
        ResBody::Once(_) => {
            return Err(Error::other("current body kind is `ResBody::Once` already"));
        }
        ResBody::Chunks(_) => {
            return Err(Error::other("current body kind is `ResBody::Chunks` already"));
        }
        ResBody::Stream(_) => {
            return Err(Error::other("current body kind is `ResBody::Stream` already"));
        }
        _ => {}
    }
    let mapped = stream.map_ok(Into::into).map_err(Into::into);
    self.body = ResBody::Stream(Box::pin(mapped));
    Ok(())
}
```

### `Piece`
`Piece` is used to write data to [`Response`].

`Piece` is simpler than [`Writer`] ant it implements [`Writer`].
```rust
pub trait Piece {
    /// Render data to [`Response`].
    fn render(self, res: &mut Response);
}
```