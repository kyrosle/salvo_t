use serde::Deserialize;

mod metadata;
pub use metadata::{Metadata, Source, SourceFrom, SourceFormat};
pub trait Extractible<'de>: Deserialize<'de> {
    fn metadata() -> &'de Metadata;
}
