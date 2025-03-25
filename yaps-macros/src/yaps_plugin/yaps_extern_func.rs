use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    TraitItemFn,
    Ident, Signature,
    ReturnType,
    visit_mut::VisitMut,
    parse,
};

use crate::utils::func_args;

#[derive(Default)]
pub struct YapsExternFuncs {
    extern_funcs: Vec<Signature>,
}

impl YapsExternFuncs {

    fn process_extern_func(&mut self, item: &TraitItemFn) -> TokenStream {
        let sig = &item.sig;
        self.extern_funcs.push(sig.clone());

        let args = func_args(sig);
        let var_idents = args.iter()
            .map(|(arg_ident, _)| {
                arg_ident
            });

        let ident = sig.ident.clone();
        let handle_ident = Ident::new(&format!("{ident}_handle"), ident.span());
        let inputs = sig.inputs.clone();
        let ret_type = match &sig.output {
            ReturnType::Default => quote!{ () },
            ReturnType::Type(_, t) => quote!{ #t },
        };

        quote! {
            fn #ident (&self, #inputs) -> ::yaps_core::Result<#ret_type> {
                let serde = self.serde();
                let args = serde.serialize((#( #var_idents ),*))?;

                let func_handle = &*self.#handle_ident.borrow();
                let result = func_handle(args)?;

                serde.deserialize(result)
            }
        }
    }

}

impl VisitMut for YapsExternFuncs {

    fn visit_token_stream_mut(&mut self, item: &mut TokenStream) {
        if let Ok(func) = parse::<TraitItemFn>(item.clone().into()) {
            if func.attrs.iter().any(|attr| {
                attr.path().is_ident("yaps_extern_func")
            }) {
                *item = self.process_extern_func(&func);
            }
        };
    }

}
