use metadata::Metadata;
use serde::Deserialize;

mod metadata;
pub trait Extractible<'de>: Deserialize<'de> {
    fn metadata()-> &'de Metadata;
}