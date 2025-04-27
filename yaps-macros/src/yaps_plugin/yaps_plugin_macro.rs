use proc_macro2::Span;
use syn::{Generics, Ident, Item, ItemImpl, ItemMod, ItemStruct, Meta, Type, parse_quote};

use darling::FromMeta;
use proc_macro_error::abort;

use super::{
    consumer_provider::{generate_consumer_impl, generate_provider_impl},
    extern_trait::*,
    wrapper::*,
    yaps_export::ExportFunc,
    yaps_extern::ExternFunc,
    yaps_impl::process_impl,
    yaps_struct::process_struct,
};

#[derive(FromMeta, Debug)]
struct YapsPluginArgs {
    pub struct_name: Option<String>,
    pub plugin_name: Option<String>,
}

fn get_plugin_struct<'a>(
    content: &'a mut [Item],
    struct_name: Option<String>,
    args_meta: &Meta, /* for diagnostic scope */
) -> &'a mut ItemStruct {
    let structs_iter = content.iter_mut().filter_map(|item| match item {
        Item::Struct(item_struct) => Some(item_struct),
        _ => None,
    });
    match struct_name {
        Some(s) => {
            let mut structs: Vec<_> = structs_iter
                .filter(|item_struct| item_struct.ident == s)
                .collect();

            match structs.len() {
                0 => abort!(args_meta, "No struct named {}", s),
                1 => structs
                    .pop()
                    .expect("Length of structs is 1, so pop expected to return a value"),
                _ => abort!(args_meta, "Multiple structs named {}", s),
            }
        }
        None => {
            let mut structs: Vec<_> = structs_iter.collect();

            match structs.len() {
                0 => abort!(args_meta, "No struct"),
                1 => structs
                    .pop()
                    .expect("Length of structs is 1, so pop expected to return a value"),
                _ => abort!(
                    args_meta,
                    "Multiple structs, consider using struct_name macro argument"
                ),
            }
        }
    }
}

fn get_plugin_impls<'a>(content: &'a mut [Item], struct_ident: &Ident) -> Vec<&'a mut ItemImpl> {
    content
        .iter_mut()
        .filter_map(|item| match item {
            Item::Impl(i) => Some(i),
            _ => None,
        })
        .filter(move |item_impl| match &*item_impl.self_ty {
            Type::Path(p) => p
                .path
                .segments
                .last()
                .is_some_and(|last_segment| &last_segment.ident == struct_ident),
            _ => false,
        })
        .collect()
}

fn generate_imports() -> Item {
    parse_quote! {
        use ::yaps_core::{
            FuncHandle as _,
        };
    }
}

pub(crate) struct YapsPluginInfo {
    pub struct_ident: Ident,
    pub struct_generics: Generics,

    pub extern_funcs_trait: Ident,
    pub wrapper_ident: Ident,

    pub plugin_name: String,

    pub export_funcs: Vec<ExportFunc>,
    pub extern_funcs: Vec<ExternFunc>,
}

impl Default for YapsPluginInfo {
    fn default() -> Self {
        YapsPluginInfo {
            struct_ident: Ident::new("NIL", Span::call_site()),
            struct_generics: Generics::default(),
            extern_funcs_trait: Ident::new("NIL", Span::call_site()),
            wrapper_ident: Ident::new("NIL", Span::call_site()),
            plugin_name: String::from("NIL"),
            export_funcs: Vec::new(),
            extern_funcs: Vec::new(),
        }
    }
}

pub(crate) fn process_yaps_module(module: &mut ItemMod, args_meta: &Meta) {
    let args = match YapsPluginArgs::from_meta(args_meta) {
        Ok(a) => a,
        Err(e) => abort!(args_meta, "Invalid yaps_plugin args: {}", e),
    };

    let content = match &mut module.content {
        Some((_, c)) => c,
        None => abort!(module, "Yaps module cannot be empty"),
    };

    let mut plugin_info = YapsPluginInfo::default();

    {
        let plugin_struct = get_plugin_struct(content, args.struct_name, args_meta);
        process_struct(plugin_struct, &mut plugin_info);
    }

    {
        let plugin_impls = get_plugin_impls(content, &plugin_info.struct_ident);
        for item in plugin_impls {
            process_impl(item, &mut plugin_info);
        }
    }

    plugin_info.plugin_name = match args.plugin_name {
        Some(n) => n,
        None => plugin_info.struct_ident.to_string(),
    };

    content.insert(0, generate_imports());

    content.push(Item::Trait(generate_extern_trait(&plugin_info)));
    content.push(Item::Impl(generate_extern_funcs_inner_impl(&plugin_info)));

    content.push(Item::Struct(generate_wrapper_struct(&mut plugin_info)));
    content.push(Item::Impl(generate_wrapper_impl(&plugin_info)));
    content.push(Item::Impl(generate_wrapper_extern_funcs_impl(&plugin_info)));

    content.push(Item::Impl(generate_provider_impl(&plugin_info)));
    content.push(Item::Impl(generate_consumer_impl(&plugin_info)));
}
