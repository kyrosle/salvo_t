# Main

Router can route http requests to different handlers.

Including mapping the url to server service and adding Middlewares etc.

Filter Part

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

`Or` , `OrElse` , `And` and `AndThen` are implemented trait `Filter`

### Filter Request (src/routing/mod.rs)

`FnFilter`
```rust
#[derive(Copy, Clone)]
#[allow(missing_debug_implementations)]
pub struct FnFilter<F>(pub F);
```
* implement `fmt::Debug` trait and `Filter` trait

#### others (src/routing/others.rs)

`MethodFilter`
Filter by request method
```rust
#[derive(Clone, PartialEq, Eq)]
pub struct MethodFilter(pub Method);
```
* implement `fmt::Debug` trait and `Filter` trait

`SchemeFilter` 
Filter by request uri scheme.
```rust
#[derive(Clone, PartialEq, Eq)]
pub struct SchemeFilter(pub Scheme, pub bool);
```
* implement `fmt::Debug` trait and `Filter` trait

`HostFilter` 
Filter by request uri host.
```rust
#[derive(Clone, PartialEq, Eq)]
pub struct HostFilter(pub String, pub bool);
```
* implement `fmt::Debug` trait and `Filter` trait

`PortFilter` Filter by request uri host.
```rust
#[derive(Clone, PartialEq, Eq)]
pub struct PortFilter(pub u16, pub bool);
```
* implement `fmt::Debug` trait and `Filter` trait

#### path (src/routing/path.rs)

Trait `PathWisp`
```rust
pub trait PathWisp: Send + Sync + fmt::Debug + 'static {
    #[doc(hidden)]
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }
    #[doc(hidden)]
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    /// Detect is that path matched.
    fn detect(&self, state: &mut PathState) -> bool;
}
```

#### Wisp Builder

__use modules__ :

`parking_lot` : 
This library provides implementations of `Mutex`, `RwLock`, `Condvar` and `Once` that are smaller, faster and more flexible than those in the Rust standard library. It also provides a ReentrantMutex type.

`regex` : Regex

`WispBuilder`
```rust
pub trait WispBuilder: Send + Sync {
    fn build(
        &self,
        name: String,
        sign: String,
        args: Vec<String>,
    ) -> Result<Box<dyn PathWisp>, String>;
}
```

`WispBuilderMap`
```rust
type WispBuilderMap = RwLock<HashMap<String, Arc<Box<dyn WispBuilder>>>>;
```


`RegexWisp`
```rust
#[derive(Debug)]
struct RegexWisp {
    name: String,
    regex: Regex,
}
```
* impl `PartialEq` , `PathWisp`

`PathWisp::detect` function:
```rust
impl PathWisp for RegexWisp {
    fn detect(&self, state: &mut PathState) -> bool {
        if self.name.starts_with('*') {
            let rest = state.all_rest();
            if rest.is_none() {
                return false;
            }
            let rest = &*rest.unwrap();

            if !rest.is_empty() || self.name.starts_with("**") {
                let cap = self.regex.captures(rest).and_then(|caps| caps.get(0));
                if let Some(cap) = cap {
                    let cap = cap.as_str().to_owned();
                    state.forward(cap.len());
                    state.params.insert(self.name.clone(), cap);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            let picked = state.pick();
            if picked.is_none() {
                return false;
            }
            let picked = picked.unwrap();
            let cap = self.regex.captures(picked).and_then(|caps| caps.get(0));
            if let Some(cap) = cap {
                let cap = cap.as_str().to_owned();
                state.forward(cap.len());
                state.params.insert(self.name.clone(), cap);
                true
            } else {
                false
            }
        }
    }
}
```

`RegexWispBuilder`
```rust
pub struct RegexWispBuilder(Regex);
impl RegexWispBuilder {
    pub fn new(checker: Regex) -> Self {
        Self(checker)
    }
}
```

`CharWispBuilder`
```rust
struct CharWisp<C> {
    name: String,
    checker: Arc<C>,
    min_width: usize,
    max_width: Option<usize>,
}
pub struct CharWispBuilder<C>(Arc<C>);
impl<C> CharWispBuilder<C> {
    pub fn new(checker: C) -> Self {
        Self(Arc::new(checker))
    }
}
```

* impl `fmt::Debug` trait and `PathWisp` trait

`PathWisp::detect` function :
```rust
impl<C> PathWisp for CharWisp<C>
where
    C: Fn(char) -> bool + Send + Sync + 'static,
{
    fn detect(&self, state: &mut PathState) -> bool {
        let picked = state.pick();
        if picked.is_none() {
            return false;
        }
        let picked = picked.unwrap();
        if let Some(max_width) = self.max_width {
            let mut chars = Vec::with_capacity(max_width);
            for ch in picked.chars() {
                if (self.checker)(ch) {
                    chars.push(ch);
                }
                if chars.len() == max_width {
                    state.forward(max_width);
                    state
                        .params
                        .insert(self.name.clone(), chars.into_iter().collect());
                    return true;
                }
            }
            if chars.len() >= self.min_width {
                state.forward(chars.len());
                state
                    .params
                    .insert(self.name.clone(), chars.into_iter().collect());
                true
            } else {
                false
            }
        } else {
            let mut chars = Vec::with_capacity(16);
            for ch in picked.chars() {
                if (self.checker)(ch) {
                    chars.push(ch);
                }
            }
            if chars.len() >= self.min_width {
                state.forward(chars.len());
                state
                    .params
                    .insert(self.name.clone(), chars.into_iter().collect());
                true
            } else {
                false
            }
        }
    }
}
```



`CombWisp`

```rust
#[derive(Debug)]
struct CombWisp(Vec<Box<dyn PathWisp>>);
impl PathWisp for CombWisp {
    fn detect(&self, state: &mut PathState) -> bool {
        let original_cursor = state.cursor;
        for child in &self.0 {
            if !child.detect(state) {
                state.cursor = original_cursor;
                return false;
            }
        }
        true
    }
}
```

`NameWisp`

```rust
#[derive(Debug, Eq, PartialEq)]
struct NameWisp(String);
impl PathWisp for NameWisp {
    fn detect(&self, state: &mut PathState) -> bool {
        if self.0.starts_with('*') {
            let rest = state.all_rest().unwrap_or_default();
            if !rest.is_empty() || self.0.starts_with("**") {
                let rest = rest.to_string();
                state.params.insert(self.0.clone(), rest);
                state.cursor.0 = state.parts.len();
                true
            } else {
                false
            }
        } else {
            let picked = state.pick();
            if picked.is_none() {
                return false;
            } 
            let picked = picked.unwrap().to_owned();
            state.forward(picked.len());
            state.params.insert(self.0.clone(), picked);
            true
        }
    }
}
```

`Construct`
```rust
#[derive(Eq, PartialEq, Debug)]
struct ConstWisp(String);
impl PathWisp for ConstWisp {
    fn detect(&self, state: &mut PathState) -> bool {
        let picked = state.pick();
        if picked.is_none() {
            return false;
        }
        let picked = picked.unwrap();
        if picked.starts_with(&self.0) {
            state.forward(self.0.len());
            true
        } else {
            false
        }
    }
}
```

```rust
struct PathParser {
    offset: usize,
    path: Vec<char>,
}
```

```rust
pub struct PathFilter {
    raw_value: String,
    path_wisps: Vec<Box<dyn PathWisp>>,
}
```
