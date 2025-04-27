use crate::defs::*;
use proc_macro_error::abort;
use quote::format_ident;
use syn::{Fields, Ident, ItemStruct, parse_quote_spanned, spanned::Spanned};

use super::yaps_plugin_macro::YapsPluginInfo;

fn extern_funcs_trait_name(struct_ident: &Ident) -> Ident {
    format_ident!("{}ExternFuncs", struct_ident)
}

pub(crate) fn process_struct(item: &mut ItemStruct, info: &mut YapsPluginInfo) {
    info.struct_ident = item.ident.clone();
    info.struct_generics = item.generics.clone();

    let extern_funcs_trait = extern_funcs_trait_name(&info.struct_ident);
    info.extern_funcs_trait = extern_funcs_trait.clone();

    let fields = match &item.fields {
        Fields::Unnamed(_) => abort!(item.fields, "yaps_plugin struct can't have unnamed fields"),
        Fields::Named(f) => f.named.iter().collect(),
        Fields::Unit => Vec::new(),
    };

    if let Some(f) = fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .find(|ident| *ident == "extern_funcs")
    {
        abort!(f, "Field named 'extern_funcs' will cause a conflict")
    }

    let fields = Fields::Named(parse_quote_spanned!(item.fields.span() =>
        {
            #( #fields, )*
            pub extern_funcs: #OnceCell<#Weak<dyn #extern_funcs_trait>>,
        }
    ));

    item.fields = fields;
}
