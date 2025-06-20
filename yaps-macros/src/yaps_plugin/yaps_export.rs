use darling::FromMeta;
use proc_macro_error::abort;
use syn::{Ident, ImplItem, ImplItemFn, ItemImpl, ReturnType, Type, parse_quote};

use crate::{
    utils::{self, FunctionArgs, parse_darling_attr},
    yaps_plugin::yaps_extern::EXTERN_ATTR,
};

pub const EXPORT_ATTR: &str = "yaps_export";

#[derive(Debug, FromMeta, Default, Clone)]
struct ExportFuncArgs {
    id: Option<String>,
    namespace: Option<String>,
}

#[derive(Debug)]
pub(crate) struct ExportFunc {
    pub is_async: bool,
    pub ident: Ident,
    pub args: FunctionArgs,
    pub ret_ty: Type,

    pub id: String,
}

pub(crate) fn process_export_funcs(item: &mut ItemImpl) -> Vec<ExportFunc> {
    let outer_args = get_outer_args(item);
    item.items
        .iter_mut()
        .filter_map(|i| process_item(i, &outer_args))
        .collect()
}

fn process_item(item: &mut ImplItem, outer_args: &Option<ExportFuncArgs>) -> Option<ExportFunc> {
    let item = match item {
        ImplItem::Fn(f) => f,
        _ => return None,
    };

    let args = utils::pop_attr(&mut item.attrs, EXPORT_ATTR)
        .as_ref()
        .map(parse_darling_attr);

    let args = merge_args(args, outer_args)?;

    Some(process_fn_item(item, args))
}

fn process_fn_item(item: &ImplItemFn, args: ExportFuncArgs) -> ExportFunc {
    if let Some(attr) = utils::get_attr(&item.attrs, EXTERN_ATTR) {
        abort!(attr, "Export function can't be extern");
    }

    match item.sig.receiver() {
        Some(r) => {
            if r.reference.is_none() || r.mutability.is_some() {
                abort!(r, "Export func must take &self");
            }
        }
        None => abort!(item.sig, "Export func must take &self"),
    };

    let ret_ty = match &item.sig.output {
        ReturnType::Type(_, t) => t.as_ref().clone(),
        _ => parse_quote! {()},
    };

    let mut id = args.id.unwrap_or(item.sig.ident.to_string());

    if let Some(namespace) = args.namespace {
        id = format!("{namespace}::{id}");
    }

    ExportFunc {
        is_async: item.sig.asyncness.is_some(),
        ident: item.sig.ident.clone(),
        args: FunctionArgs::from(&item.sig),
        ret_ty,
        id,
    }
}

fn get_outer_args(item: &mut ItemImpl) -> Option<ExportFuncArgs> {
    let outer_attrs = utils::pop_attr(&mut item.attrs, EXPORT_ATTR)?;

    if let Some(attr) = utils::get_attr(&item.attrs, EXTERN_ATTR) {
        abort!(attr, "Impl block can either be extern or export, not both")
    }

    let mut outer_args: ExportFuncArgs = parse_darling_attr(&outer_attrs);

    if outer_args.id.is_some() {
        abort!(outer_attrs, "{} on impl block cannot set id", EXPORT_ATTR)
    }

    if let Some(ref mut namespace) = outer_args.namespace {
        if namespace == "auto" {
            *namespace = get_impl_type_string(item);
        }
    }

    Some(outer_args)
}

pub fn get_impl_type_string(item: &ItemImpl) -> String {
    match item.self_ty.as_ref() {
        Type::Path(p) => p
            .path
            .get_ident()
            .expect("get_impl_type_string should only be called on plugin struct impl")
            .to_string(),
        _ => panic!("get_impl_type_string should only be called on plugin struct impl"),
    }
}

fn merge_args(
    args: Option<ExportFuncArgs>,
    outer_args: &Option<ExportFuncArgs>,
) -> Option<ExportFuncArgs> {
    let mut args = match args {
        Some(a) => a,
        None => return outer_args.clone(),
    };

    let outer_args = match outer_args {
        Some(a) => a,
        None => return Some(args),
    };

    match args.namespace {
        // Take the namespace from the outer attribute
        None => args.namespace = outer_args.namespace.clone(),
        // If the namespace is set as "" set it to None
        // This is to allow clearing namespace if it's set by the parent
        Some(ref s) => {
            if s.is_empty() {
                args.namespace = None
            }
        }
    };

    Some(args)
}
