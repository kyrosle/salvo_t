use darling::{FromField, FromMeta};
use proc_macro2::Ident;
use syn::{Attribute, Generics, Meta, NestedMeta, Type};
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
