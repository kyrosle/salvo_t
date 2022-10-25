# Part 01
This project just for me to learn a web framework which written in `rust`. Logging my every stage reading the source code.

`tokio` with features `macros` for `tokio::test` etc., rt-multi-thread

at first, I suggest the base call function like this:
```rust
fn function(&self, req: &Request, depot: &Depot, res: &mut Response) -> a result;
```
Base function modules: 
- depot.rs `Depot`
- request.rs `Request`
- response.rs `Response`


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

## TestClient
Test modules wait for request module and response module builded.

## In `Depot` 
data-struct-field:
```rust
map: HashMap<String, Box<dyn Any + Send + Sync>>,
```

## sub module `SocketAddr`
Warping the `std::net::SocketAddr` and make convert

## Request and Response

### Request module
use crate (only enum the module not the sub function or struct , enum etc.)

`cookie`
open the features : percent-encode

`hyper`
open the features : stream, server, http1, http2, tcp, client
- http::header
- http::method
- http::version
- http::{Extension, Uri}

``
- mime
- mulitimap
- once_cell
- serde

|pub|pub(crate)|pub(super)|pub(self)|
|--|--|--|--|
|for every one|for current crate|for parent modules|for current module|

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
### form module
control the file transport

### serde module
modules:

`serde::de::value::Error` as `ValError`

`serde::de::Error` as `DeError`

function:

`from_str_multi_val`

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

two macros for `Deserializer` trait
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

```

