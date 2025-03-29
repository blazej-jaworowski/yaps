use proc_macro2::Span;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse, parse_str, parse_macro_input,
    Result, Error,
    Meta, Path,
};

mod utils;
mod yaps_plugin;
mod with_serde;

use yaps_plugin::{YapsPluginInfo, YapsPluginArgs};
use with_serde::{WithSerdeInfo, WithSerdeArgs};
use darling::FromMeta;

fn attr_to_meta(path: &str, attr: TokenStream) -> Result<Meta> {
    let path: Path = parse_str(path)?;
    let attr = proc_macro2::TokenStream::from(attr);
    let attr: TokenStream = quote! {
        #path(#attr)
    }.into();
    parse(attr)
}

#[proc_macro_attribute]
pub fn with_serde(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = match attr_to_meta("with_serde", attr) {
        Ok(m) => m,
        Err(e) => return Error::new(Span::call_site(), e.to_string())
            .to_compile_error().into(),
    };

    let args = match WithSerdeArgs::from_meta(&attr) {
        Ok(a) => a,
        Err(e) => return Error::new(Span::call_site(), e.to_string())
            .to_compile_error().into(),
    };

    let with_serde_info = parse_macro_input!(item as WithSerdeInfo);

    with_serde_info.generate(args).into()
}

#[proc_macro_attribute]
pub fn yaps_plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = match attr_to_meta("with_serde", attr) {
        Ok(m) => m,
        Err(e) => return Error::new(Span::call_site(), e.to_string())
            .to_compile_error().into(),
    };

    let plugin_args = match YapsPluginArgs::from_meta(&attr) {
        Ok(a) => a,
        Err(e) => return Error::new(Span::call_site(), e.to_string())
            .to_compile_error().into(),
    };
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
pub fn yaps_extern_func(_: TokenStream, item: TokenStream) -> TokenStream {
    // This attribute is just a marker, should be removed during parsing
    let mut error: TokenStream = Error::new(
        Span::call_site(),
        "Invalid yaps_extern_func macro usage"
    ).to_compile_error().into();

    error.extend(item);
    error
}
