use proc_macro::TokenStream;
use syn::parse_macro_input;

mod utils;
mod yaps_plugin;

use yaps_plugin::{YapsPluginInfo, YapsPluginArgs};

#[proc_macro_attribute]
pub fn yaps_plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let plugin_info = parse_macro_input!(item as YapsPluginInfo);
    let plugin_args = parse_macro_input!(attr as YapsPluginArgs);

    plugin_info.generate(plugin_args).into()
}

#[proc_macro_attribute]
pub fn yaps_plugin_init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // This attribute is just a marker
    item
}

#[proc_macro_attribute]
pub fn yaps_func(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // This attribute is just a marker
    item
}
