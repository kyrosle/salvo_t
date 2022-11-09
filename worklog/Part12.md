# Main

Build proc-macro `Handler`

three proc-macro :
* function-like `macro!()`
* derive  `#[derive(Macro)]`
* attribute `#[proc_macro_attribute]`

Open the proc-macro attribute in the Cargo.toml 
```
[lib]
proc-macro = true
```

`marcos` crate files tree : 
```
└─ src
    |─ extract.rs
    |─ handler.rs
    |─ lib.rs
    └─ shared.rs
```

__Added module__ :

* `proc-macro2` : Bring proc-macro-like functionality to other contexts like build.rs and main.rs and Make procedural macros unit testable.

* `proc_macro_crate` : Providing support for `$crate` in procedural macros.

* `syn` : Parsing a stream of Rust tokens into a syntax tree of Rust source code

* `quote` : This crate provides the quote! macro for turning Rust syntax tree data structures into tokens of source code.

---

## Mainly macro : `Handler` (macros/src/lib.rs)

`handler` is a pro macro to help create `Handler` from function or impl block easily.

`Handler` is a trait, if `#[handler]` applied to `fn`,  `fn` will converted to a struct, and then implement `Handler`.

```rust
#[async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl);
}
```

After use `handler`, you don't need to care arguments' order, omit unused arguments:

```rust
#[handler]
async fn hello_world() -> &'static str {
    "Hello World"
}
```

__Statement__ : 

```rust
use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, Item};
#[proc_macro_attribute]
pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let internal = shared::is_internal(args.iter());
    let item = parse_macro_input!(input as Item);
    match handler::generate(internal, item) {
        Ok(stream) => stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
```
* `TokenStream` :  Provided by `crate::proc_macro`,representing an abstract stream of tokens, or, more specifically, a sequence of token trees.

* `AttributeArgs` : `type AttributeArgs = Vec<NestedMeta>;`

* `NestedMeta` : A enum type containing the `Meta(Meta)` and `Lit(Lit)` fields, which `Meta` means like the `Copy` in `#[derive(Copy)]`, and `Lit` means a rust literal, like `new_name` in `#[rename("new_name")]`.

* `Item` : Things that can appear directly inside of a module or scope.

* `parse_macro_input` marco : Parse the input TokenStream of a macro, triggering a compile error if the tokens fail to parse.

And then the `shred::is_internal` and the `handler::generate` will have a further explore.

# Shared (macros/src/shared.rs)

```rust
use syn::{PatType, Receiver};
pub(crate) enum InputType<'a> {
    Request(&'a PatType),
    Depot(&'a PatType),
    Response(&'a PatType),
    FlowCtrl(&'a PatType),
    UnKnown,
    Receiver(&'a Receiver),
    NoReference(&'a PatType),
    LazyExtract(&'a PatType),
}
```

* `PatType` : A type ascription pattern: `foo: f64`.

`PatType` : Data structure 
```rust
pub struct PatType {
    pub attrs: Vec<Attribute>,
    pub pat: Box<Pat>,
    pub colon_token: Colon,
    pub ty: Box<Type>,
}
```

* `Receiver` : The self argument of an associated method, whether taken by value or by reference.

__Shared Functions__ :

* `salvo_crate(internal: bool) -> syn::Ident`
```rust
use proc_macro2::{Ident, Span};
use proc_macro_crate::{crate_name, FoundCrate};

pub(crate) fn salvo_crate(internal: bool) -> syn::Ident {
    // if used in the internal crate 
    // such as `salvo_core` use this crate `salvo_macros` situation
    if internal {
        return Ident::new("crate", Span::call_site());
    }
    // otherwise get the caller original crate name form current Cargo.toml
    //
    // Ok(orig_name) if the crate was found, but not renamed in the Cargo.toml.
    //
    // Ok(RENAMED) if the crate was found, but is renamed in the Cargo.toml. 
    // RENAMED will be the renamed name.
    match crate_name("salvo") {
        Ok(salvo) => match salvo {
            FoundCrate::Itself => Ident::new("salvo", Span::call_site()),
            FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
        },
        Err(_) => match crate_name("salvo_core") {
            Ok(salvo) => match salvo {
                FoundCrate::Itself => Ident::new("salvo_core", Span::call_site()),
                FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
            },
            Err(_) => Ident::new("salvo", Span::call_site()),
        },
    }
}
```

* `crate_name(orig_name: &str) -> Result<FoundCrate, Error>` : Provided by `proc_macro_crate`, find the crate name for the given `orig_name` in the current Cargo.toml.

* `Span` : A region of source code, along with macro expansion information.

* `Span::call_site()` : The span of the invocation of the current procedural macro.
Identifiers created with this span will be resolved as if they were written directly at the macro call location (call-site hygiene) and other code at the macro call site will be able to refer to them as well.

---

* `parse_input_type(input: &FnArg) -> InputType`
```rust
use syn::{FnArg, Ident, Meta, NestedMeta, PatType, Receiver, Type, TypePath};
pub(crate) fn parse_input_type(input: &FnArg) -> InputType {
    // FnArg enum fields : `Receiver(Receiver)` and `Typed(PatType)`
    if let FnArg::Typed(p) = input {
        if let Type::Reference(ty) = &*p.ty {
            if let syn::Type::Path(nty) = &*ty.elem {
                // the last ident for path type is the real type
                // such as:
                // `::std::vec::Vec` is `Vec`
                // `Vec` is `Vec`
                let ident = &nty.path.segments.last().unwrap().ident;
                if ident == "Request" {
                    InputType::Request(p)
                } else if ident == "Response" {
                    InputType::Response(p)
                } else if ident == "Depot" {
                    InputType::Depot(p)
                } else if ident == "FlowCtrl" {
                    InputType::FlowCtrl(p)
                } else {
                    InputType::UnKnown
                }
            } else {
                InputType::UnKnown
            }
        } else if let Type::Path(nty) = &*p.ty {
            let ident = &nty.path.segments.last().unwrap().ident;
            if ident == "LazyExtract" {
                InputType::LazyExtract(p)
            } else {
                InputType::NoReference(p)
            }
        } else {
            InputType::NoReference(p)
        }
    } else if let FnArg::Receiver(r) = input {
        InputType::Receiver(r)
    } else {
        // like self on fn
        InputType::UnKnown
    }
}
```
* `FnArg` : Provided by `syn`, an argument in a function signature: the `n: usize` in `fn f(n: usize)`.

---

* `fn omit_type_path_lifetime(ty_path: &TypePath) -> TypePath` 

```rust
pub(crate) fn omit_type_path_lifetime(ty_path: &TypePath) -> TypePath {
    let reg = Regex::new(r"'\w+").unwrap();
    let ty_path = ty_path.into_token_stream().to_string();
    let ty_path = reg.replace_all(&ty_path, "'_");
    syn::parse_str(ty_path.as_ref()).unwrap()
}
```

* `TypePath` : A path like `std::slice::Iter` , optionally qualified with a self-type as in `<Vec<T> as SomeTrait>::Associated`.

---

* `fn is_internal<'a>(args: impl Iterator<Item = &'a NestedMeta>) -> bool`

check the 

```rust
pub(crate) fn is_internal<'a>(args: impl Iterator<Item = &'a NestedMeta>) -> bool {
    for arg in args {
        if matches!(arg, NestedMeta::Meta(Meta::Path(p)) if p.is_ident("internal")) {
            return true;
        }
    }
    false
}
```

* `.is_ident() -> bool` 

Equal condition :

* the path has no leading colon,
* the number of path segments is 1,
* the first path segment has no angle bracketed or parenthesized path arguments, and
* the ident of the first path segment is equal to the given one.

---
---

# Handler (macros/src/handler.rs)

```rust
pub(crate) fn generate(internal: bool, input: Item) -> syn::Result<TokenStream> {
    let salvo = salvo_crate(internal);
    match input {
        // `Fn(ItemFn)` - `ItemFn` : 
        // A free-standing function: `fn process(n: usize) -> Result<()> { ... }` .
        Item::Fn(mut item_fn) => {
            let attrs = &item_fn.attrs;
            let vis = &item_fn.vis;
            let sig = &mut item_fn.sig;
            let body = &item_fn.block;
            let name = &sig.ident;
            let docs = item_fn
                .attrs
                .iter()
                .filter(|attr| attr.path.is_ident("doc"))
                .cloned()
                .collect::<Vec<_>>();

            let sdef = quote! {
                #(#docs)*
                #[allow(non_camel_case_types)]
                #[derive(Debug)]
                #vis struct #name;
                impl #name {
                    #(#attrs)*
                    #sig {
                        #body
                    }
                }
            };

            let hfn = handle_fn(&salvo, sig)?;
            Ok(quote! {
                #sdef
                #[#salvo::async_trait]
                impl #salvo::Handler for #name {
                    #hfn
                }
            })
        }

        // `Impl(ItemImpl)` - `ItemImpl` :  
        // An impl block providing trait or associated items: `impl<A> Trait for Data<A> { ... }` .
        Item::Impl(item_impl) => {
            let mut hmtd = None;
            for item in &item_impl.items {
                if let ImplItem::Method(method) = item {
                    if method.sig.ident == Ident::new("handler", Span::call_site()) {
                        hmtd = Some(method);
                    }
                }
            }
            if hmtd.is_none() {
                return Err(syn::Error::new_spanned(
                    item_impl.impl_token,
                    "missing handle function",
                ));
            }
            let hmtd = hmtd.unwrap();
            let hfn = handle_fn(&salvo, &hmtd.sig)?;
            let ty = &item_impl.self_ty;
            let (impl_generics, ty_generics, where_clause) = &item_impl.generics.split_for_impl();

            Ok(quote! {
                #item_impl
                #[#salvo::async_trait]
                impl #impl_generics $salvo::Handler for #ty #ty_generics #where_clause {
                    #hfn
                }
            })
        }
        _ => Err(syn::Error::new_spanned(
            input,
            "#[handler] must added to `impl` or `fn`",
        )),
    }
}
```

* `Ident` : Provided by `proc-macros2`, a word of Rust code, which may be a keyword or legal variable name.

* `Item` : Provided by `proc-macros2`, things that can appear directly inside of a module or scope.


---

`fn handle_fn(salvo: &Ident, sig: &Signature) -> syn::Result<TokenStream>`

```rust
// `Signature` : A function signature in a trait or implementation: `unsafe fn initialize(&self)` .
fn handle_fn(salvo: &Ident, sig: &Signature) -> syn::Result<TokenStream> {
    let name = &sig.ident;
    let mut extract_ts = Vec::with_capacity(sig.inputs.len());
    let mut call_args: Vec<Ident> = Vec::with_capacity(sig.inputs.len());
    // parse the input arguments
    for input in &sig.inputs {
        match parse_input_type(input) {
            InputType::Request(_pat) => {
                call_args.push(Ident::new("req", Span::call_site()));
            }
            InputType::Depot(_pat) => {
                call_args.push(Ident::new("depot", Span::call_site()));
            }
            InputType::Response(_pat) => {
                call_args.push(Ident::new("res", Span::call_site()));
            }
            InputType::FlowCtrl(_pat) => {
                call_args.push(Ident::new("ctrl", Span::call_site()));
            }
            InputType::UnKnown => {
                return Err(syn::Error::new_spanned(
                    &sig.inputs,
                    "the inputs parameters must be Request, Depot, Response or FlowCtrl",
                ))
            }
            InputType::NoReference(pat) => {
                if let (Pat::Ident(ident), Type::Path(ty)) = (&*pat.pat, &*pat.ty) {
                    call_args.push(ident.ident.clone());
                    let id = &pat.pat;
                    let ty = omit_type_path_lifetime(ty);

                    extract_ts.push(quote!{
                        let #id: #ty = match req.extract().await {
                            Ok(data) => data,
                            Err(e) => {
                                #salvo::__private::tracing::error!(error = ?e, "failed to extract data");
                                res.set_status_error(#salvo::http::errors::StatusError::bad_request().with_detail(
                                    "Extract data failed"
                                ));
                                return;
                            }
                        };
                    });
                } else {
                    return Err(syn::Error::new_spanned(pat, "Invalid param definition"));
                }
            }
            InputType::LazyExtract(pat) => {
                if let (Pat::Ident(ident), Type::Path(ty)) = (&*pat.pat, &*pat.ty) {
                    call_args.push(ident.ident.clone());

                    let id = &pat.pat;
                    let ty = omit_type_path_lifetime(ty);

                    extract_ts.push(quote! {
                        let #id: #ty = #salvo::extract::LazyExtract::new();
                    });
                } else {
                    return Err(syn::Error::new_spanned(pat, "Invalid param definition"));
                }
            }
            InputType::Receiver(_) => {
                call_args.push(Ident::new("self", Span::call_site()));
            }
        }
    }

    // check signature return type
    match sig.output {
        ReturnType::Default => {
            if sig.asyncness.is_none() {
                Ok(quote!{
                    async fn handle(&self, req: &mut #salvo::Request, depot: &mut #salvo::Depot, res: &mut #salvo::Response, ctrl: &mut #salvo::routing::FlowCtrl) {
                        #(#extract_ts)*
                        Self::#name(#(#call_args),*)
                    } 
                })
            } else {
                Ok(quote!{
                    async fn handle(&self, req: &mut #salvo::Request, depot: &mut #salvo::Depot, res: &mut #salvo::Response, ctrl: &mut #salvo::routing::FlowCtrl) {
                        #(#extract_ts)*
                        Self::#name(#(#call_args),*).await
                    } 
                })
            }
        }
        ReturnType::Type(_,_ ) => {
            if sig.asyncness.is_none() {
                Ok(quote!{
                    async fn handle(&self, req: &mut #salvo::Request, depot: &mut #salvo::Depot, res: &mut #salvo::Response, ctrl: &mut #salvo::routing::FlowCtrl) {
                        #(#extract_ts)*
                        #salvo::Writer::write(Self::#name(#(#call_args),*), req, depot, res).await
                    } 
                })
            } else {
                Ok(quote!{
                    async fn handle(&self, req: &mut #salvo::Request, depot: &mut #salvo::Depot, res: &mut #salvo::Response, ctrl: &mut #salvo::routing::FlowCtrl) {
                        #(#extract_ts)*
                        #salvo::Writer::write(Self::#name(#(#call_args),*).await, req, depot, res).await
                    } 
                })
            }
        }
    }
}
```

---
---

# Extract (macros/src/extract.rs)

__add modules__ :
* `darling` : Darling is a tool for declarative attribute parsing in proc macro implementations.

```rust
struct Field {
    ident: Option<Ident>,
    ty: Type,
    sources: Vec<RawSource>,
    aliases: Vec<String>,
    rename: Option<String>,
}
```

impl `darling::FromField`

```rust
#[derive(FromMeta, Debug)]
struct RawSource {
    from: String,
    #[darling(default)]
    format: String,
}
```

```rust
struct ExtractibleArgs {
    ident: Ident,
    generics: Generics,
    fields: Vec<Field>,

    internal: bool,

    default_sources: Vec<RawSource>,
    rename_all: Option<String>
}
```

impl `darling::FromDeriveInput` 