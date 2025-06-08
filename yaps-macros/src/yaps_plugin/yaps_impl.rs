use syn::ItemImpl;

use super::{
    yaps_export::process_export_funcs, yaps_extern::process_extern_funcs,
    yaps_plugin_macro::YapsPluginInfo,
};

pub(crate) fn process_impl(item: &mut ItemImpl, info: &mut YapsPluginInfo) {
    info.export_funcs.append(&mut process_export_funcs(item));
    info.extern_funcs.append(&mut process_extern_funcs(item));
}
