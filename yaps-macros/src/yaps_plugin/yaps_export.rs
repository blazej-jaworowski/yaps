use darling::FromMeta;
use proc_macro_error::abort;
use syn::{Ident, ImplItemFn, ItemImpl, ReturnType, Type, parse_quote, visit_mut::VisitMut};

use crate::utils::{self, FunctionArgs};

#[derive(Debug, FromMeta)]
struct ExportFuncArgs {
    id: String,
}

#[derive(Debug)]
pub(crate) struct ExportFunc {
    pub is_async: bool,
    pub ident: Ident,
    pub args: FunctionArgs,
    pub ret_ty: Type,

    pub id: String,
}

#[derive(Debug, Default)]
struct ExportFuncs {
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
            Err(e) => abort!(attr, "Invalid yaps_export args: {}", e),
        };

        match item.sig.receiver() {
            Some(r) => {
                if r.reference.is_none() || r.mutability.is_some() {
                    abort!(r, "Export func must take &self")
                }
            }
            None => abort!(item.sig, "Export func must take &self"),
        };

        let ret_ty = match &item.sig.output {
            ReturnType::Type(_, t) => t.as_ref().clone(),
            _ => parse_quote! {()},
        };

        self.funcs.push(ExportFunc {
            is_async: item.sig.asyncness.is_some(),
            ident: item.sig.ident.clone(),
            args: FunctionArgs::from(&item.sig),
            ret_ty,
            id: export_args.id,
        });
    }
}

pub(crate) fn process_export_funcs(item: &mut ItemImpl) -> Vec<ExportFunc> {
    let mut export_funcs = ExportFuncs::default();
    export_funcs.visit_item_impl_mut(item);
    export_funcs.funcs
}
