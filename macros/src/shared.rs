use proc_macro2::{Ident, Span};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::ToTokens;
use regex::Regex;
use syn::{FnArg, Meta, NestedMeta, PatType, Receiver, Type, TypePath};

pub(crate) enum InputType<'a> {
    Request(&'a PatType),
    Depot(&'a PatType),
    Response(&'a PatType),
    FlowCtrl(&'a PatType),
    Unknown,
    Receiver(&'a Receiver),
    NoReference(&'a PatType),
    LazyExtract(&'a PatType),
}

pub(crate) fn salvo_crate(internal: bool) -> syn::Ident {
    if internal {
        return Ident::new("crate", Span::call_site());
    }
    match crate_name("salvo_t") {
        Ok(salvo_t) => match salvo_t {
            FoundCrate::Itself => Ident::new("salvo_t", Span::call_site()),
            FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
        },
        Err(_) => match crate_name("salvo_core") {
            Ok(salvo_t) => match salvo_t {
                FoundCrate::Itself => Ident::new("salvo_core", Span::call_site()),
                FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
            },
            Err(_) => Ident::new("salvo_t", Span::call_site()),
        },
    }
}

pub(crate) fn parse_input_type(input: &FnArg) -> InputType {
    if let FnArg::Typed(p) = input {
        if let Type::Reference(ty) = &*p.ty {
            if let syn::Type::Path(nty) = &*ty.elem {
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
                    InputType::Unknown
                }
            } else {
                InputType::Unknown
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
        InputType::Unknown
    }
}

pub(crate) fn omit_type_path_lifetimes(ty_path: &TypePath) -> TypePath {
    let reg = Regex::new(r"'\w+").unwrap();
    let ty_path = ty_path.into_token_stream().to_string();
    let ty_path = reg.replace_all(&ty_path, "'_");
    syn::parse_str(ty_path.as_ref()).unwrap()
}

pub(crate) fn is_internal<'a>(args: impl Iterator<Item = &'a NestedMeta>) -> bool {
    for arg in args {
        if matches!(arg, NestedMeta::Meta(Meta::Path(p)) if p.is_ident("internal")) {
            return true;
        }
    }
    false
}

