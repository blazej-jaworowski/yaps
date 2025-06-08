use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    Ident, ImplItem, ItemImpl, ReturnType, Signature, TraitItemFn, Type, parse_quote, parse2,
};

use crate::{defs::*, utils::parse_darling_attr};

use darling::FromMeta;
use proc_macro_error::abort;

use crate::utils::{self, FunctionArgs};

use super::yaps_export::EXPORT_ATTR;

pub const EXTERN_ATTR: &str = "yaps_extern";

#[derive(Debug, FromMeta, Default, Clone)]
struct ExternFuncArgs {
    id: Option<String>,
    namespace: Option<String>,
}

#[derive(Debug)]
pub(crate) struct ExternFunc {
    pub ident: Ident,
    pub args: FunctionArgs,
    pub ret_ty: Type,

    // We want to keep the signature unchanged
    pub sig: Signature,

    pub id: String,
}

pub(crate) fn process_extern_funcs(item: &mut ItemImpl) -> Vec<ExternFunc> {
    let outer_args = get_outer_args(item);
    item.items
        .iter_mut()
        .filter_map(|i| process_item(i, &outer_args))
        .collect()
}

fn process_item(item: &mut ImplItem, outer_args: &Option<ExternFuncArgs>) -> Option<ExternFunc> {
    let token_stream = match item {
        ImplItem::Fn(f) => f.to_token_stream(),
        ImplItem::Verbatim(ts) => ts.clone(),
        _ => return None,
    };

    let func = parse2::<TraitItemFn>(token_stream).ok()?;

    let args = utils::get_attr(&func.attrs, EXTERN_ATTR).map(parse_darling_attr);

    let args = merge_args(args, outer_args)?;

    *item = ImplItem::Verbatim(TokenStream::new());

    Some(process_fn_item(&func, args))
}

fn process_fn_item(item: &TraitItemFn, args: ExternFuncArgs) -> ExternFunc {
    if let Some(attr) = utils::get_attr(&item.attrs, EXPORT_ATTR) {
        abort!(attr, "Extern function can't be export");
    }

    if item.sig.asyncness.is_none() {
        abort!(
            item.sig,
            "Extern funcs are inherently async, you need to declare them as such"
        );
    }

    match item.sig.receiver() {
        Some(r) => {
            if r.reference.is_none() || r.mutability.is_some() {
                abort!(r, "Extern func takes &self")
            }
        }
        None => abort!(item.sig, "Extern func takes &self"),
    };

    let mut sig = item.sig.clone();

    let ret_ty = match sig.output {
        ReturnType::Type(_, ref ty) => ty.as_ref().clone(),
        ReturnType::Default => parse_quote! {()},
    };

    // Wrap the return type in the signature with Result
    sig.output = parse_quote! { -> #Result<#ret_ty> };

    let mut id = args.id.unwrap_or(item.sig.ident.to_string());

    if let Some(namespace) = args.namespace {
        id = format!("{namespace}::{id}");
    }

    ExternFunc {
        id,
        ident: item.sig.ident.clone(),
        args: FunctionArgs::from(&item.sig),
        sig,
        ret_ty,
    }
}

fn merge_args(
    args: Option<ExternFuncArgs>,
    outer_args: &Option<ExternFuncArgs>,
) -> Option<ExternFuncArgs> {
    let mut args = match args {
        Some(a) => a,
        None => return outer_args.clone(),
    };

    let outer_args = match outer_args {
        Some(a) => a,
        None => return Some(args),
    };

    match args.namespace {
        None => args.namespace = outer_args.namespace.clone(),
        Some(ref s) => {
            if s.is_empty() {
                args.namespace = None
            }
        }
    };

    Some(args)
}

fn get_outer_args(item: &mut ItemImpl) -> Option<ExternFuncArgs> {
    let outer_attrs = utils::pop_attr(&mut item.attrs, EXTERN_ATTR)?;

    if let Some(attr) = utils::get_attr(&item.attrs, EXPORT_ATTR) {
        abort!(attr, "Impl block can either be extern or export, not both")
    }

    let outer_args: ExternFuncArgs = parse_darling_attr(&outer_attrs);

    if outer_args.id.is_some() {
        abort!(outer_attrs, "{} on impl block cannot set id", EXTERN_ATTR)
    }

    Some(outer_args)
}
