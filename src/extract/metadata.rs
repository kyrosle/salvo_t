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



pub struct Source {
    pub from: SourceFrom,
    pub format: SourceFormat,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub name: &'static str,
    pub default_source: Vec<Source>,
    pub fields: Vec<Field>,
    pub rename_all: Option<RenameRule>,
}
