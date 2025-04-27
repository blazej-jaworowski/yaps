use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    Ident, ImplItem, ItemImpl, ReturnType, Signature, TraitItemFn, Type, parse_quote, parse2,
};

use crate::defs::*;

use darling::FromMeta;
use proc_macro_error::abort;

use crate::utils::{self, FunctionArgs};

#[derive(Debug, FromMeta)]
struct ExternFuncArgs {
    pub id: String,
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

// TODO: Ugly

pub(crate) fn process_extern_funcs(item: &mut ItemImpl) -> Vec<ExternFunc> {
    item.items
        .iter_mut()
        .filter_map(|item| match item {
            ImplItem::Fn(f) => Some((f.to_token_stream(), item)),
            ImplItem::Verbatim(ts) => Some((ts.clone(), item)),
            _ => None,
        })
        .filter_map(|(token_stream, item)| {
            if let Ok(f) = parse2::<TraitItemFn>(token_stream) {
                Some((f, item))
            } else {
                None
            }
        })
        .filter_map(|(f, item)| {
            let attr = utils::get_attr(&f.attrs, "yaps_extern").cloned();
            attr.map(|attr| (f, attr, item))
        })
        .map(|(f, attr, item)| {
            *item = ImplItem::Verbatim(TokenStream::new());
            (f, attr)
        })
        .map(|(func, attr)| {
            if func.sig.asyncness.is_none() {
                abort!(
                    func.sig,
                    "Extern funcs are inherently async, you need to declare them as such"
                );
            }

            let attr_args = match ExternFuncArgs::from_meta(&attr.meta) {
                Ok(a) => a,
                Err(e) => abort!(attr, "Invalid yaps_extern args: {}", e),
            };

            match func.sig.receiver() {
                Some(r) => {
                    if r.reference.is_none() || r.mutability.is_some() {
                        abort!(r, "Extern func takes &self")
                    }
                }
                None => abort!(func.sig, "Extern func takes &self"),
            };
            let args = FunctionArgs::from(&func.sig);

            let mut sig = func.sig.clone();

            let ret_ty = match sig.output {
                ReturnType::Type(_, ref mut ty) => ty.as_mut(),
                ReturnType::Default => &mut parse_quote! {()},
            };
            let extern_ret_ty = ret_ty.clone();

            // Wrap the return type in the signature with Result
            *ret_ty = parse_quote! { #Result<#extern_ret_ty> };

            ExternFunc {
                id: attr_args.id,
                ident: func.sig.ident.clone(),
                args,
                sig,
                ret_ty: extern_ret_ty,
            }
        })
        .collect()
}
