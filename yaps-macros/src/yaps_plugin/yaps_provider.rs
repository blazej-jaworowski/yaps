use proc_macro2::{TokenStream, Span};
use quote::quote;
use syn::{
    parse_quote,
    ItemImpl, Ident,
    Arm,
    LitStr,
};

use super::yaps_export::{ExportFuncs, ExportFunc};
use super::defs::*;

use crate::utils::FunctionArgs;

#[derive(Debug)]
pub struct YapsProvider {
    export_funcs: ExportFuncs,
}

impl YapsProvider {

    fn generate_match_arm(&self, func: &ExportFunc, extern_arg_funcs: &Vec<Ident>) -> Arm {
        let id_str = LitStr::new(&func.id, Span::call_site());
        let ident = &func.ident;

        let (ext_arg, args_slice) = if extern_arg_funcs.contains(ident) {
            (quote!{ &*self_clone, }, &func.args.0[1..])
        } else {
            (quote! {}, &func.args.0[0..])
        };
        let args = FunctionArgs::from(Vec::from(args_slice));

        let arg_idents = args.to_idents();
        let arg_types = args.to_types();

        parse_quote! {
            #id_str => Ok(#FunctionHandle::new(move |args| {
                let self_clone = self_clone.borrow();
                let serde = &self_clone.serde;
                let inner = &self_clone.inner;
            
                let (#arg_idents): (#arg_types) = serde.deserialize(args)?;
                let result = inner.#ident(#ext_arg #arg_idents);
                let result = serde.serialize(result)?;
            
                Ok(result)
            }))
        }
    }

    fn generate_provider_impl(&self, wrapper_ref: &Ident, extern_arg_funcs: &Vec<Ident>) -> ItemImpl {
        let provided_funcs_str = self.export_funcs.funcs.iter()
            .map(|func| LitStr::new(&func.id, Span::call_site()));

        let match_arms = self.export_funcs.funcs.iter()
            .map(|func| self.generate_match_arm(func, extern_arg_funcs));

        parse_quote! {

            impl<Data: 'static, Serde: #SerializerDeserializer<Data> + 'static> #FuncProvider<Data> for
                    #wrapper_ref<Data, Serde> {
            
                fn provided_funcs(&self) -> #YapsResult<Vec<#FunctionId>> {
                    Ok(vec![ #( #provided_funcs_str.into(), )* ])
                }
            
                fn get_func(&self, id: &#FunctionId) -> #YapsResult<#FunctionHandle<Data>> {
                    let self_clone = self.0.clone();
            
                    match id.as_str() {
                        #( #match_arms, )*
                        _ => Err(#YapsError::FunctionNotFound(id.into())),
                    }
                }
            
            }
        }
    }

    pub fn generate_code(&self, wrapper_ref: &Ident, extern_arg_funcs: &Vec<Ident>) -> TokenStream {
        let provider_impl = self.generate_provider_impl(wrapper_ref, extern_arg_funcs);
        quote! {
            #provider_impl
        }
    }

    pub fn process(item: &mut ItemImpl) -> Self {
        let export_funcs = ExportFuncs::process(item);
        YapsProvider { export_funcs }
    }

}
