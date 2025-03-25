use proc_macro::TokenStream;
use syn::parse_macro_input;

mod utils;
mod yaps_plugin;
mod with_serde;

use yaps_plugin::{YapsPluginInfo, YapsPluginArgs, YapsExternFuncInfo};
use with_serde::{WithSerdeInfo, WithSerdeArgs};

#[proc_macro_attribute]
pub fn with_serde(attr: TokenStream, item: TokenStream) -> TokenStream {
    let with_serde_args = parse_macro_input!(attr as WithSerdeArgs);
    let with_serde_info = parse_macro_input!(item as WithSerdeInfo);

    with_serde_info.generate(with_serde_args).into()
}

#[proc_macro_attribute]
pub fn yaps_plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let plugin_args = parse_macro_input!(attr as YapsPluginArgs);
    let plugin_info = parse_macro_input!(item as YapsPluginInfo);

    plugin_info.generate(plugin_args).into()
}

#[proc_macro_attribute]
pub fn yaps_init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // This attribute is just a marker
    item
}

#[proc_macro_attribute]
pub fn yaps_func(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // This attribute is just a marker
    item
}

#[proc_macro_attribute]
pub fn yaps_extern_func(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let extern_func = parse_macro_input!(item as YapsExternFuncInfo);

    extern_func.generate().into()
}
