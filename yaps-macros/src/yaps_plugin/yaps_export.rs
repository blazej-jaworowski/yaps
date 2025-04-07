use syn::{
    ItemImpl,
    ImplItemFn,
    Ident,
    visit_mut::VisitMut,
};
use darling::FromMeta;
use proc_macro_error::abort;

use crate::utils::{self, FunctionArgs};


#[derive(Debug, FromMeta)]
pub struct ExportFuncArgs {
    id: String,
}

#[derive(Debug)]
pub struct ExportFunc {
    pub is_async: bool,
    pub ident: Ident,
    pub args: FunctionArgs,
    pub id: String,
}

#[derive(Debug, Default)]
pub struct ExportFuncs {
    pub funcs: Vec<ExportFunc>,
}

impl VisitMut for ExportFuncs {

    fn visit_impl_item_fn_mut(&mut self, item: &mut ImplItemFn) {
        let attr = match utils::pop_attr(&mut item.attrs, "yaps_export") {
            Some(a) => a,
            None => return,
        };

        let export_args = match ExportFuncArgs::from_meta(&attr.meta) {
            Ok(a) => a,
            Err(_) => abort!(attr, "Invalid yaps_export args"),
        };

        self.funcs.push(ExportFunc {
            is_async: item.sig.asyncness.is_some(),
            ident: item.sig.ident.clone(),
            args: FunctionArgs::from(&item.sig),
            id: export_args.id,
        });
    }

}

impl ExportFuncs {

    pub fn process(item: &mut ItemImpl) -> Self {
        let mut export_funcs = Self::default();
        export_funcs.visit_item_impl_mut(item);
        export_funcs
    }

}
