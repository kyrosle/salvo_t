use cruet::Inflector;

use self::RenameRule::*;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
#[non_exhaustive]
pub enum SourceFrom {
    Param,
    Query,
    Header,
    Cookie,
    Body,
    Request,
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
#[non_exhaustive]
pub enum SourceFormat {
    MultiMap,
    Json,
    Request,
}

impl FromStr for SourceFrom {
    type Err = crate::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "param" => Ok(Self::Param),
            "query" => Ok(Self::Query),
            "header" => Ok(Self::Header),
            "cookie" => Ok(Self::Cookie),
            "body" => Ok(Self::Body),
            "request" => Ok(Self::Request),
            _ => Err(crate::Error::Other(
                format!("invalid source from `{}`", input).into(),
            )),
        }
    }
}
/// Rename rule for a field.
#[allow(clippy::enum_variant_names)]
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
impl FromStr for RenameRule {
    type Err = crate::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        for (name, rule) in RENAME_RULES {
            if input == *name {
                return Ok(*rule);
            }
        }
        Err(crate::Error::other(format!(
            "invalid rename rule: {}",
            input
        )))
    }
}
impl FromStr for SourceFormat {
    type Err = crate::Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "multimap" => Ok(Self::MultiMap),
            "json" => Ok(Self::Json),
            "request" => Ok(Self::Request),
            _ => Err(crate::Error::Other("invalid source format".into())),
        }
    }
}

impl RenameRule {
    pub fn rename(&self, name: impl AsRef<str>) -> String {
        let name = name.as_ref();
        match *self {
            PascalCase => name.to_pascal_case(),
            LowerCase => name.to_lowercase(),
            UpperCase => name.to_uppercase(),
            CamelCase => name.to_camel_case(),
            SnakeCase => name.to_snake_case(),
            ScreamingSnakeCase => SnakeCase.rename(name).to_ascii_uppercase(),
            KebabCase => SnakeCase.rename(name).replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase.rename(name).replace('_', "-"),
        }
    }
}

static RENAME_RULES: &[(&str, RenameRule)] = &[
    ("lowercase", LowerCase),
    ("UPPERCASE", UpperCase),
    ("PascalCase", PascalCase),
    ("camelCase", CamelCase),
    ("snake_case", SnakeCase),
    ("SCREAMING_SNAKE_CASE", ScreamingSnakeCase),
    ("kebab-case", KebabCase),
    ("SCREAMING-KEBAB-CASE", ScreamingKebabCase),
];

#[derive(Debug, Clone, Copy)]
pub struct Source {
    pub from: SourceFrom,
    pub format: SourceFormat,
}

impl Source {
    pub fn new(from: SourceFrom, format: SourceFormat) -> Self {
        Self { from, format }
    }
}

/// Information about struct field.
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
impl Field {
    pub fn new(name: &'static str) -> Self {
        Self::with_sources(name, vec![])
    }
    pub fn with_sources(name: &'static str, sources: Vec<Source>) -> Self {
        Self {
            name,
            sources,
            aliases: vec![],
            rename: None,
            metadata: None,
        }
    }
    pub fn metadata(mut self, metadata: &'static Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
    pub fn set_alias(mut self, aliases: Vec<&'static str>) -> Self {
        self.aliases = aliases;
        self
    }
    pub fn add_alias(mut self, alias: &'static str) -> Self {
        self.aliases.push(alias);
        self
    }
    pub fn rename(mut self, rename: &'static str) -> Self {
        self.rename = Some(rename);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub name: &'static str,
    pub default_source: Vec<Source>,
    pub fields: Vec<Field>,
    pub rename_all: Option<RenameRule>,
}
impl Metadata {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            default_source: vec![],
            fields: Vec::with_capacity(0),
            rename_all: None,
        }
    }
    pub fn set_default_sources(mut self, default_sources: Vec<Source>) -> Self {
        self.default_source = default_sources;
        self
    }
    pub fn set_fields(mut self, fields: Vec<Field>) -> Self {
        self.fields = fields;
        self
    }
    pub fn add_default_source(mut self, source: Source) -> Self {
        self.default_source.push(source);
        self
    }
    pub fn add_field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }
    pub fn rename_all(mut self, rename_all: RenameRule) -> Self {
        self.rename_all = Some(rename_all);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_source_from() {
        for (key, value) in [
            ("param", SourceFrom::Param),
            ("query", SourceFrom::Query),
            ("header", SourceFrom::Header),
            ("cookie", SourceFrom::Cookie),
            ("body", SourceFrom::Body),
            ("request", SourceFrom::Request),
        ] {
            assert_eq!(key.parse::<SourceFrom>().unwrap(), value);
        }
        assert!("abcd".parse::<SourceFrom>().is_err());
    }

    #[test]
    fn test_parse_source_format() {
        for (key, value) in [
            ("multimap", SourceFormat::MultiMap),
            ("json", SourceFormat::Json),
            ("request", SourceFormat::Request),
        ] {
            assert_eq!(key.parse::<SourceFormat>().unwrap(), value);
        }
        assert!("abcd".parse::<SourceFormat>().is_err());
    }

    #[test]
    fn test_parse_rename_rule() {
        for (key, value) in RENAME_RULES {
            assert_eq!(key.parse::<RenameRule>().unwrap(), *value);
        }
        assert!("abcd".parse::<RenameRule>().is_err());
    }

    #[test]
    fn test_rename_rule() {
        assert_eq!(PascalCase.rename("rename_rule"), "RenameRule");
        assert_eq!(LowerCase.rename("RenameRule"), "renamerule");
        assert_eq!(UpperCase.rename("rename_rule"), "RENAME_RULE");
        assert_eq!(CamelCase.rename("RenameRule"), "renameRule");
        assert_eq!(SnakeCase.rename("RenameRule"), "rename_rule");
        assert_eq!(ScreamingSnakeCase.rename("rename_rule"), "RENAME_RULE");
        assert_eq!(KebabCase.rename("rename_rule"), "rename-rule");
        assert_eq!(ScreamingKebabCase.rename("rename_rule"), "RENAME-RULE");
    }
}
