mod extract;
mod handler;
mod shared;

use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, Item};

#[proc_macro_attribute]
pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let internal = shared::is_internal(args.iter());
    let item = parse_macro_input!(input as Item);
    todo!()
}
