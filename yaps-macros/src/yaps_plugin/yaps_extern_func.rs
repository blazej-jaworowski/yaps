use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    TraitItemFn,
    Ident, Signature, Attribute,
    ReturnType,
    visit_mut::VisitMut,
    parse,
};

use crate::utils::func_args;

fn is_extern_func_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("yaps_extern_func")
}

#[derive(Debug, Clone)]
pub struct YapsExternFunc {
    pub sig: Signature,
    pub handle_ident: Ident,

    pub plugin_name: String,
    pub func_name: String,
}

#[derive(Default, Debug, Clone)]
pub struct YapsExternFuncs {
    pub extern_funcs: Vec<YapsExternFunc>,
}

impl YapsExternFuncs {

    fn get_func_key(attrs: impl Iterator<Item = Attribute>) -> (String, String) {
        // TODO
        // let attr = attrs.filter(is_extern_func_attr).last().unwrap();
        // dbg!(attr);
        ("dupa".into(), "blada".into())
    }

    fn process_extern_func(&mut self, item: &TraitItemFn) -> TokenStream {
        let (plugin_name, func_name) = Self::get_func_key(item.attrs.iter().cloned());
        let sig = &item.sig;

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

        self.extern_funcs.push(YapsExternFunc {
            sig: sig.clone(),
            handle_ident: handle_ident.clone(),
            plugin_name,
            func_name,
        });

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
            if func.attrs.iter().cloned()
                .filter(is_extern_func_attr)
                .count() == 1
            {
                *item = self.process_extern_func(&func);
            }
        };
    }

}
