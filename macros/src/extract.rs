use darling::{FromDeriveInput, FromField, FromMeta};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, Error, Generics, Meta, NestedMeta, Type};

use crate::shared::is_internal;
struct Field {
    ident: Option<Ident>,
    ty: Type,
    sources: Vec<RawSource>,
    aliases: Vec<String>,
    rename: Option<String>,
}

#[derive(FromMeta, Debug)]
struct RawSource {
    from: String,
    #[darling(default)]
    format: String,
}

impl FromField for Field {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let ident = field.ident.clone();
        let attrs = field.attrs.clone();
        let sources = parse_sources(&attrs, "source")?;
        Ok(Self {
            ident,
            ty: field.ty.clone(),
            sources,
            aliases: parse_aliases(&field.attrs)?,
            rename: parse_rename(&field.attrs)?,
        })
    }
}

struct ExtractibleArgs {
    ident: Ident,
    generics: Generics,
    fields: Vec<Field>,

    internal: bool,

    default_sources: Vec<RawSource>,
    rename_all: Option<String>,
}

impl FromDeriveInput for ExtractibleArgs {
    fn from_derive_input(input: &syn::DeriveInput) -> darling::Result<Self> {
        let ident = input.ident.clone();
        let generics = input.generics.clone();
        let attrs = input.attrs.clone();
        let default_sources = parse_sources(&attrs, "default_source")?;
        let data = match &input.data {
            syn::Data::Struct(data) => data,
            _ => {
                return Err(Error::new_spanned(
                    ident,
                    "Extractible can only be applied to an struct.",
                )
                .into());
            }
        };
        let mut fields = Vec::with_capacity(data.fields.len());
        for field in data.fields.iter() {
            fields.push(Field::from_field(field)?);
        }
        let mut internal = false;
        for attr in &attrs {
            if attr.path.is_ident("extract") {
                if let Meta::List(list) = attr.parse_meta()? {
                    if is_internal(list.nested.iter()) {
                        internal = true;
                        break;
                    }
                }
            }
        }
        Ok(Self {
            ident,
            generics,
            fields,
            internal,
            default_sources,
            rename_all: parse_rename_rule(&input.attrs)?,
        })
    }
}

static RENAME_RULES: &[(&str, &str)] = &[
    ("lowercase", "LowerCase"),
    ("UPPERCASE", "UpperCase"),
    ("PascalCase", "PascalCase"),
    ("camelCase", "CamelCase"),
    ("snake_case", "SnakeCase"),
    ("SCREAMING_SNAKE_CASE", "ScreamingSnakeCase"),
    ("kebab-case", "KebabCase"),
    ("SCREAMING-KEBAB-CASE", "ScreamingKebabCase"),
];

fn metadata_rename_rule(salvo: &Ident, input: &str) -> Result<TokenStream, Error> {
    let mut rule = None;
    for (name, value) in RENAME_RULES {
        if input == *name {
            rule = Some(*value);
        }
    }
    match rule {
        Some(rule) => {
            let rule = Ident::new(rule, Span::call_site());
            Ok(quote! {
                #salvo::extract::metadata::RenameRule::#rule
            })
        }
        None => {
            Err(Error::new_spanned(input,
                "Invalid rename rule, valid rules are: lowercase, UPPERCASE, PascalCase, camelCase, snake_case, SCREAMING_SNAKE_CASE, kebab-case, SCREAMING-KEBAB-CASE",
            ))
        }
    }
}

fn metadata_source(salvo: &Ident, source: &RawSource) -> TokenStream {
    let from = Ident::new(&source.from.to_pascal_case(), Span::call_site());
}
fn parse_sources(attrs: &[Attribute], key: &str) -> darling::Result<Vec<RawSource>> {
    let mut sources = Vec::with_capacity(4);
    for attr in attrs {
        if attr.path.is_ident("extract") {
            if let Meta::List(list) = attr.parse_meta()? {
                for meta in list.nested.iter() {
                    if matches!(meta, NestedMeta::Meta(Meta::List(item)) if item.path.is_ident(key))
                    {
                        let mut source: RawSource = FromMeta::from_nested_meta(meta)?;
                        if source.format.is_empty() {
                            if source.from == "request" {
                                source.format = "request".to_string();
                            } else {
                                source.format = "multimap".to_string();
                            }
                        }

                        if !["request", "param", "query", "header", "body"]
                            .contains(&source.from.as_str())
                        {
                            return Err(darling::Error::custom(format!(
                                "source from is invalid: {}",
                                source.from
                            )));
                        }

                        if !["multimap", "json", "request"].contains(&source.from.as_str()) {
                            return Err(darling::Error::custom(format!(
                                "source from is invalid: {}",
                                source.from
                            )));
                        }

                        if source.from == "request" && source.format != "request" {
                            return Err(darling::Error::custom(
                                "source format mut be `request` sources",
                            ));
                        }
                        sources.push(source);
                    }
                }
            }
        }
    }
    Ok(sources)
}
