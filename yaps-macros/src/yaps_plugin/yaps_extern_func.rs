use proc_macro2::{TokenStream, Span};
use quote::quote;
use syn::{
    Result, Error,
    TraitItemFn,
    ItemImpl,
    Ident, Attribute,
    ReturnType,
    visit_mut::VisitMut,
    parse,
};
use darling::FromMeta;

use crate::utils::func_args;

fn is_extern_func_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("yaps_extern_func")
}

#[derive(Debug, FromMeta)]
pub struct YapsExternFuncArgs {
    plugin: String,
    func: String,
}

#[derive(Debug, Clone)]
pub struct YapsExternFunc {
    pub handle_ident: Ident,

    pub plugin_name: String,
    pub func_name: String,
}

#[derive(Default, Debug, Clone)]
pub struct YapsExternFuncs {
    pub extern_funcs: Vec<YapsExternFunc>,
}

impl YapsExternFuncs {

    pub fn process(item: &mut ItemImpl) -> Self {
        let mut yaps_extern_funcs = YapsExternFuncs::default();
        yaps_extern_funcs.visit_item_impl_mut(item);
        yaps_extern_funcs
    }

    fn get_func_key(attrs: impl Iterator<Item = Attribute>) -> Result<(String, String)> {
        let attr = attrs.filter(is_extern_func_attr).last()
            .ok_or(Error::new(Span::call_site(), "No yaps_extern_func attribute"))?;
        let args = YapsExternFuncArgs::from_meta(&attr.meta)?;
        Ok((args.plugin, args.func))
    }

    fn process_extern_func(&mut self, item: &TraitItemFn) -> TokenStream {
        let (plugin_name, func_name) = match Self::get_func_key(item.attrs.iter().cloned()) {
            Ok(r) => r,
            Err(e) => return Error::new(Span::call_site(), e.to_string())
                .to_compile_error(),
        };
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
            if func.attrs.iter()
                .filter(|f| is_extern_func_attr(f))
                .count() == 1
            {
                *item = self.process_extern_func(&func);
            }
        };
    }

}
