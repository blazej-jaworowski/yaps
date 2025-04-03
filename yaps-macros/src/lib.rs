use proc_macro::TokenStream;
use syn::{
    parse_macro_input,
    ItemImpl,
};
use proc_macro_error::proc_macro_error;


#[macro_use]
mod utils;
mod yaps_plugin;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn yaps_plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut impl_block = parse_macro_input!(item as ItemImpl);
    yaps_plugin::process_yaps_plugin(&mut impl_block).into()
}
