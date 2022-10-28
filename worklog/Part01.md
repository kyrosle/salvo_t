# Part 01
`tokio` with features `macros` for `tokio::test` etc., `rt-multi-thread` 

At first, I suggest the base call function like this as follow and start from here.
```rust
fn function(&self, req: &Request, depot: &Depot, res: &mut Response) -> a result;
```
Base function modules: 
- depot.rs `Depot`
- request.rs `Request`
- response.rs `Response`

---
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
#[derive(Default)]
pub struct Depot {
    map: HashMap<String, Box<dyn Any + Send + Sync>>,
}
```
---
## SocketAddr (src/addr.rs)
Warping the `std::net::SocketAddr` and make convert

---
## Request and Response
### form (src/http/form.rs)
control the file transport

The extracted text fields and uploaded files from a `multipart/form-data` request.

#### `FilePart`
__Used modules__ :  

`tempfile`: This crate provides several approaches to creating temporary files and directories.

`textnonce` : Using `TextNonce` , for cryptographic concept of an arbitrary number that is never used more than once.

A file that is to be inserted into a `multipart/*` or alternatively an uploaded file that
was received as part of `multipart/*` parsing.
```rust
#[derive(Clone, Debug)]
pub struct FilePart {
    name: Option<String>,
    /// The headers of the part
    headers: HeaderMap,
    /// A temporary file containing the file content
    path: PathBuf,
    /// Optionally, the size of the file.  This is filled when multiparts are parsed, but is
    /// not necessary when they are generated.
    size: Option<usize>,
    // The temporary directory the upload was put into, saved for the Drop trait
    temp_dir: Option<PathBuf>,
}
```
__main function__ : 
Create a new temporary FilePart 
(when created this way, the file will be deleted once the FilePart object goes out of scope).
```rust
pub async fn create(field: &mut Field<'_>) -> Result<FilePart, ParseError> {
    let mut path =
        tokio::task::spawn_blocking(|| [tempfile]Builder::new().prefix("salvo_http_multipart").tempdir())
            .await
            .expect("Runtime spawn blocking poll error")?
            .into_path();

    let temp_dir = Some(path.clone());
    let name = field.file_name().map(|s| s.to_owned());
    path.push(format!(
        "{}.{}",
        TextNonce::sized_urlsafe(32).unwrap().into_string(),
        name.as_deref()
            .and_then(|name| { Path::new(name).extension().and_then(OsStr::to_str) })
            .unwrap_or("unknown")
    ));
    let mut file = File::create(&path).await?;
    while let Some(chunk) = field.chunk().await? {
        file.write_all(&chunk).await?;
    }
    Ok(FilePart {
        name,
        headers: field.headers().to_owned(),
        path,
        size: None,
        temp_dir,
    })
}
```
If `FilePart` was dropped, clean the file and dir path :
```rust
impl Drop for FilePart {
    fn drop(&mut self) {
        if let Some(temp_dir) = &self.temp_dir {
            let path = self.path.clone();
            let temp_dir = temp_dir.to_owned();
            tokio::task::spawn_blocking(move || {
                std::fs::remove_file(&path).ok();
                std::fs::remove_dir(temp_dir).ok();
            });
        }
    }
}
```

#### `FormData`
```rust
#[derive(Debug)]
pub struct FormData {
    /// Name-value pairs for plain text fields. Technically, these are form data parts with no
    /// filename specified in the part's `Content-Disposition`.
    pub fields: MultiMap<String, String>,
    /// Name-value pairs for temporary files. Technically, these are form data parts with a filename
    /// specified in the part's `Content-Disposition`.
    pub files: MultiMap<String, FilePart>,
}
```
__main function__ : 
Parse MIME `multipart/*` information from a stream as a [`FormData`].
```rust
[FormData]
pub(crate) async fn read(headers: &HeaderMap, body: ReqBody) -> Result<FormData, ParseError> {
    match headers.get(CONTENT_TYPE) {
        Some(ctype) if ctype == "application/x-www-form-urlencoded" => {
            let data = hyper::body::to_bytes(body)
                .await
                .map(|d| d.to_vec())
                .map_err(ParseError::Hyper)?;
            let mut form_data = FormData::new();
            form_data.fields = form_urlencoded::parse(&data).into_owned().collect();
            Ok(form_data)
        },
        Some(ctype) if ctype.to_str().unwrap_or("").starts_with("multipart/") => {
            let mut form_data = FormData::new();
            if let Some(boundary) = headers.get(CONTENT_TYPE)
            .and_then(|ct|ct.to_str().ok())
            .and_then(|ct| multer::parse_boundary(ct).ok()) {
                let mut multipart = Multipart::new(body, boundary);
                while let Some(mut field) = multipart.next_field().await? {
                    if let Some(name) = field.name().map(|s|s.to_owned()) {
                        if field.headers().get(CONTENT_TYPE).is_some() {
                            form_data.files.insert(name, FilePart::create(&mut field).await?);
                        } else {
                            form_data.fields.insert(name, field.text().await?);
                        }
                    }
                }
            }
            Ok(form_data)
        }
        _ => Err(ParseError::InvalidContentType)
    }
}
```



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
- `I`:`Into<Cow<'de, str>>` 

`from_str_map(I)`
- `I`: `IntoIterator<Item = (K, V)> + 'de`,
- `T`: `Deserialize<'de>`,
- `K`: `Into<Cow<'de, str>>`,
- `V`: `Into<Cow<'de, str>>`,

`from_str_multi_map(I)`
- `I`: `IntoIterator<Item = (K, C)> + 'de`,
- `T`: `Deserialize<'de>`,
- `K`: `Into<Cow<'de, str>> + Hash + std::cmp::Eq + 'de`,
- `C`: `IntoIterator<Item = V> + 'de`,
- `V`: `Into<Cow<'de, str>> + std::cmp::Eq + 'de`,


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

Deserialize for request:
```rust
pub(crate) struct RequestDeserializer<'de> {
    params: &'de HashMap<String, String>,
    queries: &'de MultiMap<String, String>,
    cookies: &'de cookie::CookieJar,
    headers: &'de HeaderMap,
    payload: Option<Payload<'de>>,
    metadata: &'de Metadata,
    field_index: isize,
    field_source: Option<&'de Source>,
    field_str_value: Option<&'de str>,
    field_vec_value: Option<Vec<CowValue<'de>>>,
}
```
`Payload` field:
```rust
#[derive(Debug, Clone)]
pub(crate) enum Payload<'a> {
    FormData(&'a FormData),
    JsonStr(&'a str),
    JsonMap(HashMap<&'a str, &'a RawValue>)
}
```
#### `request` 
For helping `Request` Deserializer.
##### `Payload`
```rust
#[derive(Debug, Clone)]
pub(crate) enum Payload<'a> {
    FormData(&'a FormData),
    JsonStr(&'a str),
    JsonMap(HashMap<&'a str, &'a RawValue>),
}
```

##### `RequestDeserializer`
data struct:

impl `de::Deserializer` and `de::MapAccess` trait

`de::MapAccess` : Provides a Visitor access to each entry of a map in the input.

```rust
#[derive(Debug)]
pub(crate) struct RequestDeserializer<'de> {
    params: &'de HashMap<String, String>,
    queries: &'de MultiMap<String, String>,
    cookies: &'de cookie::CookieJar,
    headers: &'de HeaderMap,
    payload: Option<Payload<'de>>,
    metadata: &'de Metadata,
    field_index: isize,
    field_source: Option<&'de Source>,
    field_str_value: Option<&'de str>,
    field_vec_value: Option<Vec<CowValue<'de>>>,
}
```
main functions:

```rust
// Construct from Request and Metadata
[RequestDeserializer]
pub(crate) fn new(
    request: &'de mut Request,
    metadata: &'de Metadata,
) -> Result<RequestDeserializer<'de>, ParseError> {
    let mut payload = None;
    if let Some(ctype) = request.content_type() {
        match ctype.subtype() {
            mime::WWW_FORM_URLENCODED | mime::FORM_DATA => {
                payload = request.form_data.get().map(Payload::FormData);
            }
            mime::JSON => {
                if let Some(data) = request.payload.get() {
                    payload = match serde_json::from_slice::<HashMap<&str, &RawValue>>(data) {
                        Ok(map) => Some(Payload::JsonMap(map)),
                        Err(_) => Some(Payload::JsonStr(std::str::from_utf8(data)?)),
                    };
                }
            }
            _ => {}
        }
    }
    Ok(RequestDeserializer {
        params: request.params(),
        queries: request.queries(),
        cookies: request.cookies(),
        headers: request.headers(),
        payload,
        metadata,
        field_index: -1,
        field_source: None,
        field_str_value: None,
        field_vec_value: None,
    })
}
```
```rust
[RequestDeserializer]
fn deserialize_value<T>(&mut self, seed: T) -> Result<T::Value, ValError>
where T: de::DeserializeSeed<'de>
{
    let source = self.field_source.take().expect("MapAccess::next_value called before next_key");

    if source.from == SourceFrom::Body && source.format == SourceFormat::Json {
        let value = self.field_str_value.expect("MapAccess::next_value called before next_key");
        let mut value = serde_json::Deserializer::new(serde_json::de::StrRead::new(value));
        seed.deserialize(&mut value).map_err(|_| ValError::custom("pare value error"))
    } else if source.from == SourceFrom::Request {
        let field = self.metadata.fields.get(self.field_index as usize) .expect("Field must exist");
        let metadata = field.metadata.expect("Field's metadata must exist");
        seed.deserialize(RequestDeserializer {
            params: self.params,
            queries: self.queries,
            headers: self.headers,
            cookies: self.cookies,
            payload: self.payload.clone(),
            metadata,
            field_index: -1,
            field_source: None,
            field_str_value: None,
            field_vec_value: None,
        })
    } else if let Some(value) = self.field_str_value.take() {
        seed.deserialize(CowValue(value.into()))
    } else if let Some(value) = self.field_vec_value.take() {
        seed.deserialize(VecValue(value.into_iter()))
    } else {
        Err(ValError::custom("parse value error"))
    }
}
```

```rust
[RequestDeserializer]
fn next(&mut self) -> Option<Cow<'_, str>> {
    if self.field_index < self.metadata.fields.len() as isize -1 {
        self.field_index += 1;
        let field = &self.metadata.fields[self.field_index as usize]; 
        let sources = if !field.sources.is_empty() {
            &field.sources
        } else if !self.metadata.default_source.is_empty() {
            &self.metadata.default_source
        } else {
            tracing::error!("no sources for field {}", field.name);
            return None;
        };

        self.field_str_value = None;
        self.field_vec_value = None;
        let field_name: Cow<'_, str> = if let Some(rename_all) = self.metadata.rename_all {
            if let Some(rename) = field.rename {
                Cow::from(rename)
            } else {
                rename_all.rename(field.name).into()
            }
        } else {
            if let Some(rename) = field.rename {
                rename
            } else {
                field.name
            }
            .into()
        };

        for source in sources {
            match source.from {
                SourceFrom::Request => {
                    self.field_source = Some(source);
                    return Some(Cow::from(field.name));
                }
                SourceFrom::Param => {
                    let mut value = self.params.get(&*field.name);
                    if value.is_none() {
                        for alias in  &field.aliases {
                            value = self.params.get(*alias);
                            if value.is_some() {
                                break;
                            }
                        }
                    }
                    if let Some(value) = value {
                        self.field_str_value = Some(value);
                        self.field_source = Some(source);
                        return Some(Cow::from(field.name));
                    }
                }
                SourceFrom::Query => {
                    let mut value = self.queries.get_vec(field.name.as_ref());
                    if value.is_none() {
                        for alias in &field.aliases {
                            value = if self.queries.get_vec(*alias);
                            if value.is_some() {
                                break;
                            }
                        }
                    }
                    if let Some(value) = value {
                        self.field_vec_value = Some(value.iter().map(|v| CowValue(v.into())).collect());
                        self.field_source = Some(source);
                        return Some(Cow::from(field.name));
                    }
                }
                SourceFrom::Header => {
                    let mut value = None;
                    if self.headers.contains_key(field_name.as_ref()) {
                        value = Some(self.headers.get_all(field.name.as_ref()));
                    } else {
                        for alias in &field.aliases {
                            if self.headers.contains_key(*alias) {
                                value = Some(self.headers.get_all(*alias));
                                break;
                            }
                        }
                    };
                    if let Some(value) = value {
                        self.field_vec_value = Some(value.iter().map(|v| CowValue(Cow::from(v.to_str().unwrap_or_default()))).collect());
                        self.field_source = Some(source);
                        return Some(Cow::from(field.name))
                    }
                }
                SourceFrom::Cookie => {
                    let mut value = None;
                    if let Some(cookie) = self.cookies.get(field.name.as_ref()) {
                        value = Some(cookie.value());
                    } else {
                        for alias in &field.aliases {
                            if let Some(cookie) = self.cookies.get(*alias) {
                                value = Some(cookie.value());
                                break;
                            }
                        }
                    };
                    if let Some(value) = value {
                        self.field_str_value = Some(value);
                        self.field_source = Some(source);
                        return Some(Cow::from(field.name));
                    }
                }
                SourceFrom::Body => match source.format {
                    SourceFormat::Json => {
                        if let Some(payload) = &self.payload {
                            match payload {
                                Payload::FormData(form_data) => {
                                    let mut value = form_data.fields.get(field_name.as_ref());
                                    if value.is_none() {
                                        for alias in &field.aliases {
                                            value = form_data.fields.get(*alias);
                                            if value.is_some() {
                                                break;
                                            }
                                        }
                                    }
                                    if let Some(value) = value {
                                        self.field_str_value = Some(value);
                                        self.field_source = Some(source);
                                        return Some(Cow::from(field.name));
                                    }else {
                                        return None;
                                    }
                                }
                                Payload::JsonMap(ref map) => {
                                    let mut value = map.get(field_name.as_ref());
                                    if value.is_none() {
                                        for alias in &field.aliases {
                                            value = map.get(alias);
                                            if value.is_some() {
                                                break;
                                            }
                                        }
                                    }
                                    if let Some(value) = value {
                                        self.field_str_value = Some(value.get());
                                        self.field_source = Some(source);
                                        return Some(Cow::from(field.name));
                                    } else {
                                        return None;
                                    }
                                }
                                Payload::JsonStr(value) => {
                                    self.field_str_value = Some(*value);
                                    self.field_source = Some(source);
                                    return Some(Cow::from(field.name));
                                }
                            }
                        } else {
                            return None;
                        }
                    }
                    SourceFormat::MultiMap => {
                        if let Some(Payload::FormData(form_data)) = self.payload {
                            let mut value = form_data.fields.get_vec(field.name);
                            if value.is_none() {
                                for alias in &field.aliases {
                                    value = form_data.fields.get_vec(*alias);
                                    if value.is_some() {
                                        break;
                                    }
                                }
                            }
                            if let Some(value) = value {
                                self.field_vec_value = Some(value.iter().map(|v| CowValue(Cow::from(v))).collect());
                                self.field_source = Some(source);
                                return Some(Cow::from(field.name));
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    }
                    _ => {
                        panic!("Unsupported source format: {:?}", source.format);
                    }
                }

            }
        }
    }
    None
    }
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

### Extract (src/extract)
let you deserialize request to custom type

#### Metadata (src/extract/metadata)
data struct:
```rust
/// Struct's metadata information.
#[derive(Clone, Debug)]
pub struct Metadata {
    /// The name of this type.
    pub name: &'static str,
    /// Default sources of all fields.
    pub default_sources: Vec<Source>,
    /// Fields of this type.
    pub fields: Vec<Field>,
    /// Rename rule for all fields of this type.
    pub rename_all: Option<RenameRule>,
}
```
---
##### Source
```rust
/// Request source for extract data.
#[derive(Copy, Clone, Debug)]
pub struct Source {
    /// The source from.
    pub from: SourceFrom,
    /// the origin data format of the field.
    pub format: SourceFormat,
}
```
`SourceFrom` and `SourceFormat` both impl the `FromStr` trait

###### `SourceFrom`
Source from for a field.
```rust
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[non_exhaustive]
pub enum SourceFrom {
    /// The field will extracted from url param.
    Param,
    /// The field will extracted from url query.
    Query,
    /// The field will extracted from http header.
    Header,
    /// The field will extracted from http cookie.
    #[cfg(feature = "cookie")]
    Cookie,
    /// The field will extracted from http payload.
    Body,
    /// The field will extracted from request.
    Request,
}
```

###### `SourceFormat`
Source format for a source. This format is just means that field format, not the request mime type.

For example, the request is posted as form, but if the field is string as json format, it can be parsed as json.
```rust
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[non_exhaustive]
pub enum SourceFormat {
    /// MulitMap format. This is the default.
    MultiMap,
    /// Json format.
    Json,
    /// Request format means this field will extract from the request.
    Request,
}
```

###### `RenameRule`
Rename rule for a field.

__Using module__ `cruet` : Adds String based inflections for Rust. Snake, kebab, train, camel, sentence, class, and title cases as well as ordinalize, deordinalize, demodulize, deconstantize, and foreign key are supported as both traits and pure functions acting on String types.

```rust
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub enum RenameRule {
    /// Rename direct children to "lowercase" style.
    LowerCase,
    /// Rename direct children to "UPPERCASE" style.
    UpperCase,
    /// Rename direct children to "PascalCase" style, as typically used for
    /// enum variants.
    PascalCase,
    /// Rename direct children to "camelCase" style.
    CamelCase,
    /// Rename direct children to "snake_case" style, as commonly used for
    /// fields.
    SnakeCase,
    /// Rename direct children to "SCREAMING_SNAKE_CASE" style, as commonly
    /// used for constants.
    ScreamingSnakeCase,
    /// Rename direct children to "kebab-case" style.
    KebabCase,
    /// Rename direct children to "SCREAMING-KEBAB-CASE" style.
    ScreamingKebabCase,
}
```

###### `Field`
Information about struct field.
```rust
#[derive(Clone, Debug)]
pub struct Field {
    /// Field name.
    pub name: &'static str,
    /// Field sources.
    pub sources: Vec<Source>,
    /// Field aliaes.
    pub aliases: Vec<&'static str>,
    /// Field rename.
    pub rename: Option<&'static str>,
    /// Field metadata. This is used for nested extractible types.
    pub metadata: Option<&'static Metadata>,
}
```



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

`serde_json` for its `RawValue`

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
    pub(crate) form_data: tokio::sync::OnceCell<FormData>,
    pub(crate) payload: tokio::sync::OnceCell<Vec<u8>>,

    version: Version,
    pub(crate) remote_addr: Option<SocketAddr>,
}
```

__functions__ :

Read field with `T(take)`, `&T(&)`, `&mut T(&mut)` types functions.

`from_data(&mut self) -> Result<&FormData, ParseError>` :
read from `self.headers` get the `content_type` as `ctype`
and then match the `ctype` 
to construct the `FormData` with the `body` in `self.form_data`.

`file<'a>(&'a mut self, key: &'a str) -> Option<'a FilePart>` : 
read file data from the `self.form_data`.

`payload(&mut self) -> Result<&Vec<u8>, ParseError>` : 
read from body like `json` etc.

`extract_with_metadata<'de, T>(&'de mut self, metadata: &'de Metadata) -> Result<T, ParseError>` : 
use `from_request(self, metadata)` to get from `self.form_data` and `self.payload()`. 

Parsing Self Value

`pub fn accept(&self) -> Vec<Mime>` Get Accept

---
### Response (src/http/response.rs)

#### `ResBody`
Response body type.
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
impl `Stream` trait like a futures state machine.
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

#### `Response`
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