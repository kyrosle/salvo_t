use serde::Deserialize;

mod metadata;
pub use metadata::{Metadata, Source};
pub trait Extractible<'de>: Deserialize<'de> {
    fn metadata() -> &'de Metadata;
}
