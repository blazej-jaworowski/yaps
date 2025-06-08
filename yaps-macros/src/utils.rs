use darling::FromMeta;
use proc_macro_error::abort;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, FnArg, Ident, Meta, Pat, Signature, Token, Type, parse_quote, punctuated::Punctuated,
};

#[derive(Debug, Clone)]
pub struct FunctionArgs(pub Vec<(Ident, Type)>);

impl From<&Signature> for FunctionArgs {
    fn from(sig: &Signature) -> Self {
        let args_vec = sig
            .inputs
            .iter()
            .filter_map(|input| match input {
                FnArg::Typed(t) => Some(t),
                _ => None,
            })
            .filter_map(|pat_type| {
                match &*pat_type.pat {
                    // TODO: add support
                    //Pat::Wild(_) => { /* ... */ },
                    Pat::Ident(pat_ident) => {
                        let arg_ident = pat_ident.ident.clone();
                        let arg_type = (*pat_type.ty).clone();
                        Some((arg_ident, arg_type))
                    }
                    _ => None,
                }
            })
            .collect();
        Self(args_vec)
    }
}

impl From<Vec<(Ident, Type)>> for FunctionArgs {
    fn from(vec: Vec<(Ident, Type)>) -> Self {
        Self(vec)
    }
}

impl FunctionArgs {
    pub fn to_idents(&self) -> Punctuated<Ident, Token![,]> {
        let idents = self.0.iter().map(|(ident, _)| ident);
        parse_quote! { #( #idents ),* }
    }

    pub fn to_types(&self) -> Punctuated<Type, Token![,]> {
        let types = self.0.iter().map(|(_, ty)| ty);
        parse_quote! { #( #types ),* }
    }
}

pub fn punctuated_into_tuple<T: ToTokens>(mut p: Punctuated<T, Token![,]>) -> TokenStream {
    if !p.empty_or_trailing() {
        p.push_punct(parse_quote! {,});
    }

    quote! { (#p) }
}

pub fn get_attr<'a>(attributes: &'a [Attribute], ident: &str) -> Option<&'a Attribute> {
    attributes.iter().find(|attr| attr.path().is_ident(ident))
}

pub fn pop_attr(attributes: &mut Vec<Attribute>, ident: &str) -> Option<Attribute> {
    let attr = match get_attr(attributes, ident) {
        Some(a) => a.clone(),
        None => return None,
    };

    attributes.retain(|attr| !attr.path().is_ident(ident));

    Some(attr)
}

pub fn parse_darling_attr<A: FromMeta + Default>(attr: &Attribute) -> A {
    match attr.meta {
        Meta::Path(_) => A::default(),
        Meta::List(_) => match A::from_meta(&attr.meta) {
            Ok(a) => a,
            Err(e) => abort!(attr, "Invalid attribute args: {}", e),
        },
        _ => abort!(attr, "Invalid attribute usage"),
    }
}
