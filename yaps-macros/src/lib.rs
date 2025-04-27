// We allow this because of defs usage (didn't find a better solution, should be ok)
#![allow(clippy::borrow_interior_mutable_const)]
mod defs;

#[macro_use]
mod utils;
mod yaps_plugin;

use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::{
    parse_macro_input,
    ItemMod,
    Meta,
    spanned::Spanned,
};
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn yaps_plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut module = parse_macro_input!(item as ItemMod);

    let attr: proc_macro2::TokenStream = attr.into();
    let attr = quote_spanned! { attr.span() => yaps_plugin(#attr) }.into();

    let args = parse_macro_input!(attr as Meta);
    yaps_plugin::process_yaps_module(&mut module, &args);
    quote_spanned! { module.span() => #module }.into()
}
