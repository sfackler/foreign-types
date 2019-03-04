extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::parse::Input;

mod build;
mod parse;

/// ```
/// foreign_type! {
///     /// This is a thing!
///     pub type Thing: Sync + Send {
///         type CType = thing_sys::thing;
///         fn drop = thing_sys::thing_free;
///         fn clone = thing_sys::thing_dup;
///     }
/// }
/// ```
#[proc_macro]
pub fn foreign_type_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);
    build::build(input).into()
}
