# Main

Router can route http requests to different handlers.

Including mapping the url to server service and adding Middlewares etc.

## Filter (src/routing/filter/mod.rs)
`Filter` trait for filter request.
```rust
pub trait Filter: fmt::Debug + Send + Sync + 'static {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Create a new filter use ```And``` filter.
    fn and<F>(self, other: F) -> And<Self, F>
    where
        Self: Sized,
        F: Filter + Send + Sync,
    {
        And {
            first: self,
            second: other,
        }
    }

    /// Create a new filter use ```Or``` filter.
    fn or<F>(self, other: F) -> Or<Self, F>
    where
        Self: Sized,
        F: Filter + Send + Sync,
    {
        Or {
            first: self,
            second: other,
        }
    }

    /// Create a new filter use ```AndThen``` filter.
    fn and_then<F>(self, fun: F) -> AndThen<Self, F>
    where
        Self: Sized,
        F: Fn(&mut Request, &mut PathState) -> bool + Send + Sync + 'static,
    {
        AndThen {
            filter: self,
            callback: fun,
        }
    }

    /// Create a new filter use ```OrElse``` filter.
    fn or_else<F>(self, fun: F) -> OrElse<Self, F>
    where
        Self: Sized,
        F: Fn(&mut Request, &mut PathState) -> bool + Send + Sync + 'static,
    {
        OrElse {
            filter: self,
            callback: fun,
        }
    }

    /// Filter ```Request``` and returns false or true.
    fn filter(&self, req: &mut Request, path: &mut PathState) -> bool;
}
```

### PathState

__use modules__ :

`percent_encoding` : URLs use special characters to indicate the parts of the request. like :
```
"foo <bar>" -> "foo%20%3Cbar%3E"
```

```rust
pub type PathParams = HashMap<String, String>;

#[derive(Debug, Eq, PartialEq)]
pub struct PathState {
    pub(crate) parts: Vec<String>,
    pub(crate) cursor: (usize, usize), // 0: parts index,  1: parts[index].ok_pos..last
    pub(crate) params: PathParams,
    pub(crate) end_slash: bool, // For rest match, we want include the last slash.
}
```

__Assisted Function__ : 
```rust
fn decode_url_path_safely(path: &str) -> String {
    percent_encoding::percent_decode_str(path)
        .decode_utf8_lossy()
        .to_string()
}
```
__Functions__ :

New from url_path string :
```rust
[PathState]
pub fn new(url_path: &str) -> Self {
    let end_slash = url_path.ends_with('/');
    let parts = url_path
        .trim_start_matches('/')
        .trim_end_matches('/')
        .split('/')
        .filter_map(|p| {
            if !p.is_empty() {
                Some(decode_url_path_safely(p))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    PathState {
        parts,
        cursor: (0, 0),
        params: PathParams::new(),
        end_slash,
    }
}
```

Pick :

```rust
[PathState]
pub fn pick(&self) -> Option<&str> {
    match self.parts.get(self.cursor.0) {
        None => None,
        Some(part) => {
            if self.cursor.1 >= part.len() {
                let row = self.cursor.0 + 1;
                self.parts.get(row).map(|s| &**s)
            } else {
                Some(&part[self.cursor.1..])
            }
        }
    }
}
```

Get all rest : 

```rust
[PathState]
pub fn all_rest(&self) -> Option<Cow<'_, str>> {
    if let Some(picked) = self.pick() {
        if self.cursor.0 > self.parts.len() {
            if self.end_slash {
                Some(Cow::Owned(format!("{}/", picked)))
            } else {
                Some(Cow::Borrowed(picked))
            }
        } else {
            let last = self.parts[self.cursor.0 + 1..].join("/");
            if self.end_slash {
                Some(Cow::Owned(format!("{}/{}/", picked, last)))
            } else {
                Some(Cow::Owned(format!("{}/{}", picked, last)))
            }
        }
    } else {
        None
    }
}
```

others : 

```rust
pub fn forward(&mut self, steps: usize) {
    let mut steps = steps + self.cursor.1;
    while let Some(part) = self.parts.get(self.cursor.0) {
        if part.len() > steps {
            self.cursor.1 = steps;
            return;
        } else {
            steps -= part.len();
            self.cursor = (self.cursor.0 + 1, 0);
        }
    }
}
pub fn ended(&self) -> bool {
    self.cursor.0 >= self.parts.len()
}
```

### opts (src/routing/filter/opts.rs)

#### Or
```rust
pub struct Or<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}
```

#### OrElse
```rust
pub struct OrElse<T, F> {
    pub(super) filter: T,
    pub(super) callback: F,
}
```

#### And
```rust
pub struct And<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}
```

#### AndThen
```rust
pub struct AndThen<T, F> {
    pub(super) filter: T,
    pub(super) callback: F,
}

```

## Router (src/routing/router.rs)