use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse,
    parse_quote,
    ItemImpl, ImplItemFn,
    TraitItemFn,
    ReturnType,
    Ident, Type,
    visit_mut::VisitMut,
};

use darling::FromMeta;
use proc_macro_error::abort;

use crate::utils::{self, FunctionArgs};


#[derive(Debug, FromMeta)]
struct ExternFuncArgs {
    pub id: String,
}


#[derive(Debug)]
pub struct ExternFunc {
    pub ident: Ident,
    pub args: FunctionArgs,
    pub ret_type: Type,
    pub id: String,
}

#[derive(Debug, Default)]
pub struct ExternFuncs {
    pub funcs: Vec<ExternFunc>,
}

impl ExternFuncs {

    pub fn process(item: &mut ItemImpl) -> Self {
        let mut extern_funcs = ExternFuncs::default();
        extern_funcs.visit_item_impl_mut(item);
        extern_funcs
    }

}

impl VisitMut for ExternFuncs {

    fn visit_impl_item_fn_mut(&mut self, item: &mut ImplItemFn) {
        let mut stream = item.to_token_stream();
        self.visit_token_stream_mut(&mut stream);
        // TODO: We need to remove this node somehow
    }

    fn visit_token_stream_mut(&mut self, item: &mut TokenStream) {
        let func: TraitItemFn = match parse(item.clone().into()) {
            Ok(i) => i,
            Err(_) => return,
        };

        let attr = match utils::get_attr(&func.attrs, "yaps_extern") {
            Some(a) => a,
            None => return,
        };

        let attr_args = match ExternFuncArgs::from_meta(&attr.meta) {
            Ok(a) => a,
            Err(_) => abort!(attr, "Invalid yaps_extern args"),
        };

        let args = FunctionArgs::from(&func.sig);
        let ret_type = match func.sig.output {
            ReturnType::Type(_, ty) => *ty,
            ReturnType::Default => parse_quote!(()),
        };

        self.funcs.push(ExternFunc {
            id: attr_args.id,
            ident: func.sig.ident,
            args,
            ret_type,
        });

        *item = TokenStream::new();
    }

}
