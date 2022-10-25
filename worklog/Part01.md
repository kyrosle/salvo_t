# Part 01
`tokio` with features `macros` for `tokio::test` etc., rt-multi-thread

At first, I suggest the base call function like this as follow and start from here.
```rust
fn function(&self, req: &Request, depot: &Depot, res: &mut Response) -> a result;
```
Base function modules: 
- depot.rs `Depot`
- request.rs `Request`
- response.rs `Response`
---
## Delay Trait
`Any` : A trait to emulate dynamic typing.
```rust
pub fn inject<V: Any + Send + Sync>(&mut self, value: V) -> &mut Self {
    self.map
        .insert(format!("{:?}", TypeId::of::<V>()), Box::new(value));
    self
}
```
`Send` : safe to send it to another thread.

`Sync` : safe to share between threads (T is Sync if and only if &T is Send).

---
## TestClient
Test modules wait for request module and response module builded.

---
## Depot (src/http/depot.rs)
data-struct-field:
```rust
map: HashMap<String, Box<dyn Any + Send + Sync>>,
```
---
## SocketAddr (src/addr.rs)
Warping the `std::net::SocketAddr` and make convert

---
## Request and Response
### form (src/http/form.rs)
control the file transport

---
### serde (src/serde)
modules:

`serde::de::value::Error` as `ValError`

`serde::de::Error` as `DeError`

Having function:

`from_str_multi_val(I)`

- `I`:`IntoIterator<Item = C> + 'de` 

- `C`:`Into<Cow<'de, str>> + std::cmp::Eq + 'de`

`from_str_val(I)` 
- I:`Into<Cow<'de, str>>` 

```rust
// impl `IntoDeserializer` trait
// impl `Deserializer` trait
struct CowValue<'de>(Cow<'de, str>);

// impl `IntoDeserializer` trait
// impl `Deserializer` trait
struct VecValue<I>(I);

// impl `EnumAccess` trait
struct ValueEnumAccess<'de>(Cow<'de, str>);
// impl `VariantAccess` trait
struct UnitOnlyVariantAccess;
```

`EnumAccess` : Provides a Visitor access to the data of an enum in the input.

`VariantAccess` : Called when deserializing a variant with no values.

two self define macros for `Deserializer` trait
```rust
macro_rules! forward_cow_parsed_value {
    ($($ty:ident => $method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
            {
                match self.0.parse::<$ty>() {
                    Ok(val) => val.into_deserializer().$method(visitor),
                    Err(e) => Err(DeError::custom(e))
                }
            }
        )*
    }
}
macro_rules! forward_vec_parsed_value {
    ($($ty:ident => $method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
            {
                if let Some(item) = self.0.into_iter().next() {
                    match item.0.parse::<$ty>() {
                        Ok(val) => val.into_deserializer().$method(visitor),
                        Err(e) => Err(DeError::custom(e))
                    }
                } else {
                    Err(DeError::custom("expected vec not empty"))
                }
            }
        )*
    }
}
use serde::de::forward_to_deserialize_any;
```
---
### Error (src/error) 
__Ignoring using cfg(anyhow)__

use `std::io::Error` as `IoError`

use `serde::de::Error` as `DeError`

use `std::error::Error` as `StdError`

use crate `anyhow` for `anyhow::Error` Warping

use type `BoxedError = Box<dyn std::error::Error + Send + Sync>`

use crate `thiserror` for its `#[error("message")]` usage

use crate `multer` : An async parser for multipart/form-data content-type in Rust.

```rust
// Error is from module error.rs () and http/error.rs (parse_error and status_error)
pub type Result<T> = std::result::Result<T, Error>;
```

main `Error` struct :
```rust
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Hyper(hyper::Error),
    HttpParse(ParseError),
    HttpStatus(StatusError),
    Io(IoError),
    SerdeJson(serde_json::Error),
    Anyhow(anyhow::Error),
    Other(BoxedError),
}
```

delay modules `writer` in src/writer

delay module `error` in src/http/error

trait `From<Infallible>` 

---

`ParseError` and `StatusError` were impl the trait `Writer` with function:
```rust
async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response);
```
---
### ParseError

For `ParseError` enum the type of Error when using Parse such as Parsing Body, Parsing Url, using Parsing Serde and etc.

---
### StatusError

`StatusError` struct: 
```rust
pub struct StatusError {
    pub code: StatusCode,
    pub name: String,
    pub summary: Option<String>,
    pub detail: Option<String>,
}
```

Impl `std::fmt::Display` trait

Impl such type error `From<T>`

`From<Infallible>` mean never return type

Function `from_code(StatusCode) -> Option<StatusError>` 

Having two pub self-functions `with_summary` and `with_detail` were design as chained call.

Define a convenience macro_rules for `StatusCode` numbers (enum type)
```rust
macro_rules! default_errors {
    ($($sname:ident, $code:expr, $name:expr, $summary:expr);+) => {
        $(
            /// Create a new `StatusError`.
            pub fn $sname() -> StatusCode {
                StatusError {
                    code: $code,
                    name: $name.into(),
                    summary: Some(summary.into()),
                    detail: None,
                }
            }
        )+
    };
}
```
---
### Writer (src/writer)
A trait is able to write the `ParseError` or `StatusError` into the `Response` part.

---
### Request (src/http/request.rs)
use crate (only enum the module not the sub function or struct , enum etc.)

`cookie`
open the features : percent-encode

`hyper`
open the features : stream, server, http1, http2, tcp, client
- http::header
- http::method
- http::version
- http::{Extension, Uri}

`mime`
Media Type
- mime

`multimap`
wrapper around `std::collections::HashMap` 
but allow same key,
just like the value is a vector.
- multimap

`once_cell`
just like the `lazy_static`
- once_cell

`serde` Serializer and Deserializer
- serde

`form_urlencoded` : Convert a byte string in the `application/x-www-form-urlencoded` syntax into a `iterator` of `(name, value)` pairs (collected as `HashMap` such as).
- form_urlencoded

| pub           | pub(crate)        | pub(super)         | pub(self)          |
| ------------- | ----------------- | ------------------ | ------------------ |
| for every one | for current crate | for parent modules | for current module |

main struct: 
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
    // pub(crate) form_data: tokio::sync::OnceCell<FormData>,
    pub(crate) payload: tokio::sync::OnceCell<Vec<u8>>,

    version: Version,
    pub(crate) remote_addr: Option<SocketAddr>,
}
```

In Request implementation:

`pub fn accept(&self) -> Vec<Mime>` Get Accept