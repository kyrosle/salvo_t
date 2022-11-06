# Main
res object:
* src/http/range.rs
* src/writer/json.rs && redirect.rs && text.rs

## Range (src/http/range.rs)

HTTP Range header representation.
```rust
#[derive(Clone, Debug, Copy)]
pub struct HttpRange {
    /// Start position.
    pub start: u64,
    /// Total length.
    pub length: u64,
}
```

Parses Range HTTP header string as per RFC 2616.

* `header` is HTTP Range header (e.g. `bytes=bytes=0-9`).
* `size` is full size of response (file).

`fn parse(header: &str, size: u64) -> Result<Vec<HttpRange>, ParseError>`

---

## Writer 

### Json (src/writer/json.rs)
Write serializable content to response as json content.

It will set ```content-type``` to ```application/json; charset=utf-8```.
```rust
pub struct Json<T>(pub T);
```

impl `Piece` trait

### Redirect (src/writer/redirect.rs)

Response that redirects the request to another location.

```rust
#[derive(Clone, Debug)]
pub struct Redirect {
    status_code: StatusCode,
    location: HeaderValue,
}
```

__Functions__ :

`fn other(uri: Impl TryInto<hyper::http::Uri>) -> Self`

Create a new `Redirect` that uses a [`303 See Other`][mdn] status code.

This redirect instructs the client to change the method to GET for the subsequent request
to the given `uri`, which is useful after successful form submission, file upload or when
you generally don't want the redirected-to page to observe the original request method and
body (if non-empty).

__Panics__

If `uri` isn't a valid `Uri`.

[mdn]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/303

---

`fn temporary(uri: Impl TryInto<hyper::http::Uri>) -> Self`

Create a new `Redirect` that uses a [`307 Temporary Redirect`][mdn] status code.

__Panics__

If `uri` isn't a valid `Uri`.

[mdn]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/307

---

`fn permanent(uri: Impl TryInto<hyper::http::Uri>) -> Self`

Create a new `Redirect` that uses a [`308 Permanent Redirect`][mdn] status code.

__Panics__

If `uri` isn't a valid `Uri`.

[mdn]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/308

---

`fn found(uri: Impl TryInto<hyper::http::Uri>) -> Self`

Create a new `Redirect` that uses a [`302 Found`][mdn] status code.

This is the same as `Redirect::temporary`, except the status code is older and thus
supported by some legacy applications that doesn't understand the newer one, but some of
those applications wrongly apply `Redirect::other` (`303 See Other`) semantics for this
status code. It should be avoided where possible.

__Panics__

If `uri` isn't a valid `Uri`.

[mdn]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/303

---

* impl `Piece` trait.

### Text (src/writer/text.rs)

Write text content to response as text content.

```rust
#[non_exhaustive]
pub enum Text<C> {
    /// It will set `content-type` to `text/plain; charset=utf-8`.
    Plain(C),
    /// It will set `content-type` to `application/json; charset=utf-8`.
    Json(C),
    /// It will set `content-type` to `application/xml; charset=utf-8`.
    Xml(C),
    /// It will set `content-type` to `text/html; charset=utf-8`.
    Html(C),
    /// It will set `content-type` to `text/javascript; charset=utf-8`.
    Js(C),
    /// It will set `content-type` to `text/css; charset=utf-8`.
    Css(C),
    /// It will set `content-type` to `text/csv; charset=utf-8`.
    Csv(C),
    /// It will set `content-type` to `application/atom+xml; charset=utf-8`.
    Atom(C),
    /// It will set `content-type` to `application/rss+xml; charset=utf-8`.
    Rss(C),
    /// It will set `content-type` to `application/rdf+xml; charset=utf-8`.
    Rdf(C),
}
```

__Functions__ : 

`fn set_header(self, res: &mut Response) -> C`
* `C: AsRef<str>`

impl `Piece` for `Text<&'static str>`

impl `Piece` for `Text<String>`

impl `Piece` for `Text<&'a String>`