# Main
Prepare for the `Request` , `Response` and `Depot`

__use module(s)__ : 
* `tokio` with features `macros` for `tokio::test` etc., `rt-multi-thread` 

At first, We can suggest the `base call function` like this (Similar with web `go`)
as follow and the following will start from this suggestion.
```rust
fn function(&self, req: &Request, depot: &Depot, res: &mut Response) -> a result;
```
Base function params: 
- `Depot` from `depot.rs `
- `Request` from `request.rs`
- `Response` from `response.rs`

---
__encounter__ : 

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

# Preparation before preparation

## TestClient
Test modules wait for request module and response module builded.

## SocketAddr (src/addr.rs)
Warping the `std::net::SocketAddr` and make convert

# Preparation of Building Request and Response

`FilePart` ,`FormData` from `from.rs`


## form (src/http/form.rs)
__Main usage__ :  Control the file transport

* The extracted text fields and uploaded files from a `multipart/form-data` request.

### `FilePart`
__Used modules__ :  

* `tempfile` : This crate provides several approaches to creating temporary files and directories.

* `textnonce` : Using `TextNonce` , for cryptographic concept of an arbitrary number that is never used more than once.

A file that is to be inserted into a `multipart/*` or alternatively an uploaded file that
was received as part of `multipart/*` parsing.

Data struct :

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

* Create a new temporary `FilePart`

* When created this way, the file will be deleted once the FilePart object goes out of scope.
```rust
[FilePart]
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

### `FormData`

Data struct : 

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

* Parse MIME `multipart/*` information from a stream as a [`FormData`].

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
## serde (src/serde/mod.rs)

__Used error__ :

* `serde::de::value::Error` as `ValError`

* `serde::de::Error` as `DeError`

* `serde::de::VariantAccess` : 
Provides a Visitor access to the data of an enum in the input.

* `serde::de::EnumAccess` : 
Called when deserializing a variant with no values.

__Having function__ :

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

Metadata struct : 

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

Two self define macros for `Deserializer` trait , convenience  
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
metadata :
        bool => deserialize_bool,
        u8 => deserialize_u8,
        u16 => deserialize_u16,
        u32 => deserialize_u32,
        u64 => deserialize_u64,
        i8 => deserialize_i8,
        i16 => deserialize_i16,
        i32 => deserialize_i32,
        i64 => deserialize_i64,
        f32 => deserialize_f32,
        f64 => deserialize_f64,

use serde::de::forward_to_deserialize_any;
metadata :
        char
        str
        string
        unit
        bytes
        byte_buf
        unit_struct
        tuple_struct
        struct
        identifier
        tuple
        ignored_any
        seq
        map
```

### `request` (src/serde/request.rs) 
For helping `Request` Deserializer.

#### `Payload`

`Payload` field:

```rust
#[derive(Debug, Clone)]
pub(crate) enum Payload<'a> {
    FormData(&'a FormData),
    JsonStr(&'a str),
    JsonMap(HashMap<&'a str, &'a RawValue>),
}
```

#### `RequestDeserializer`

Data struct:

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

__Implements__ : 

`de::Deserializer` trait and `de::MapAccess` trait

* `de::MapAccess` : Provides a Visitor access to each entry of a map in the input.

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

__Main functions__ :

Construct from Request and Metadata
```rust
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
## Error (src/error/mod.rs) 
use `std::io::Error` as `IoError`

use `serde::de::Error` as `DeError`

use `std::error::Error` as `StdError`

use crate `anyhow` for `anyhow::Error` Warping

use type `BoxedError = Box<dyn std::error::Error + Send + Sync>`

use crate `thiserror` for its `#[error("message")]` usage

use crate `multer` : An async parser for multipart/form-data content-type in Rust.

Type define :
```rust
//Error is from module error.rs () and http/error.rs (parse_error and status_error)
pub type Result<T> = std::result::Result<T, Error>;
```

Main `Error` struct :
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

__Implements__ : 

Different Error `From<E>` trait for this `Error`

* trait `From<Infallible>` 

`Writer` trait from src/writer

`ParseError` struct and `StatusError` struct from src/http/error

`ParseError` and `StatusError` were Implemented the trait `Writer` with function:
```rust
async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response);
```
---
### ParseError (src/http/errors/parse_error.rs)

For `ParseError` enum the type of Error when using Parse such as Parsing Body, Parsing Url, using Parsing Serde and etc.
```rust
/// ParseError, errors happened when read data from http request.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ParseError {
    /// The Hyper request did not have a valid Content-Type header.
    #[error("The Hyper request did not have a valid Content-Type header.")]
    InvalidContentType,
    /// The Hyper request's body is empty.
    #[error("The Hyper request's body is empty.")]
    EmptyBody,
    /// Parse error when parse from str.
    #[error("Parse error when parse from str.")]
    ParseFromStr,
    /// Parse error when parse from str.
    #[error("Parse error when decode url.")]
    UrlDecode,
    /// Deserialize error when parse from request.
    #[error("Deserialize error.")]
    Deserialize(#[from] DeError),
    /// DuplicateKey.
    #[error("DuplicateKey.")]
    DuplicateKey,
    /// The Hyper request Content-Type top-level Mime was not `Multipart`.
    #[error("The Hyper request Content-Type top-level Mime was not `Multipart`.")]
    NotMultipart,
    /// The Hyper request Content-Type sub-level Mime was not `FormData`.
    #[error("The Hyper request Content-Type sub-level Mime was not `FormData`.")]
    NotFormData,
    /// InvalidRange.
    #[error("InvalidRange")]
    InvalidRange,
    /// An multer error.
    #[error("Multer error: {0}")]
    Multer(#[from] multer::Error),
    /// An I/O error.
    #[error("I/O error: {}", _0)]
    Io(#[from] IoError),
    /// An error was returned from hyper.
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    /// An error occurred during UTF-8 processing.
    #[error("UTF-8 processing error: {0}")]
    Utf8(#[from] Utf8Error),
    /// Serde json error.
    #[error("Serde json error: {0}")]
    SerdeJson(#[from] serde_json::error::Error),
    /// Custom error that does not fall under any other error kind.
    #[error("Other error: {0}")]
    Other(BoxedError),
}

/// Result type with `ParseError` has it's error type.
pub type ParseResult<T> = Result<T, ParseError>;
```

---
### StatusError (src/http/statusError.rs)

`StatusError` struct: 
```rust
pub struct StatusError {
    pub code: StatusCode,
    pub name: String,
    pub summary: Option<String>,
    pub detail: Option<String>,
}
```

__Implements__ : 

`std::fmt::Display` trait

Different errors `From<Error>`

* `From<Infallible>` mean never return type

__Function__ :

`from_code(StatusCode) -> Option<StatusError>` :

* Create new `StatusError` with code.
If code is not error, it will be `None`.

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
### Writer (src/writer/mod.rs)
A trait is able to write the `ParseError` , `StatusError`
and `Result<(Writer + Send),(Writer + Send)>`
into the `Response` part.

`Writer` is used to write data to response.
```rust
#[async_trait]
pub trait Writer {
    /// Write data to [`Response`].
    #[must_use = "write future must be used"]
    async fn write(mut self, req: &mut Request, depot: &mut Depot, res: &mut Response);
}
```
Impl `Writer` for `&'static str` and `String`

etc. like this: 
```rust
#[async_trait]
impl<T, E> Writer for Result<T, E>
where
    T: Writer + Send,
    E: Writer + Send,
{
    async fn write(mut self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        match self {
            Ok(v) => v.write(req, depot, res).await,
            Err(e) => e.write(req, depot, res).await,
        }
    }
}
#[async_trait]
impl Writer for Error {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let status_error = match self {
            Error::HttpStatus(e) => e,
            _ => StatusError::internal_server_error(),
        };
        res.set_status_error(status_error);
    }
}

#[async_trait]
impl Writer for anyhow::Error {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.set_status_error(StatusError::internal_server_error());
    }
}
```

### Extract (src/extract/metadata.rs)
let you deserialize request to custom type.

#### Metadata
Struct's metadata information.

Data struct:
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
#### Source
Request source for extract data.

Data struct : 
```rust
#[derive(Copy, Clone, Debug)]
pub struct Source {
    /// The source from.
    pub from: SourceFrom,
    /// the origin data format of the field.
    pub format: SourceFormat,
}
```

__Implements__ : 

`SourceFrom` and `SourceFormat` both impl the `FromStr` trait

#### `SourceFrom`
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

#### `SourceFormat`
Source format for a source. This format is just means that field format, not the request mime type.

For example, the request is posted as form, but if the field is string as json format, it can be parsed as json.

Data struct : 
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

#### `RenameRule`
Rename rule for a field.

__Using module__
* `cruet` : Adds String based inflections for Rust. 
 Snake, kebab, train, camel, sentence, class, and title cases as well as ordinalize, deordinalize, demodulize, deconstantize, and foreign key are supported as both traits and pure functions acting on String types.

Data struct :
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

#### `Field`
Information about struct field.

Data struct :
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


