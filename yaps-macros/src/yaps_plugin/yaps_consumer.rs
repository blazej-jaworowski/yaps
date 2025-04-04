use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote,
    Token,
    Ident, Stmt, LitStr,
    ItemImpl, ImplItemFn,
    ItemTrait, TraitItemFn,
    FieldValue,
    punctuated::Punctuated,
    spanned::Spanned,
};

use super::yaps_extern::ExternFuncs;
use super::yaps_extern_arg::ExternArgs;

use super::defs::*;
use crate::utils;


pub struct YapsConsumer {
    extern_trait_ident: Ident,
    extern_funcs: ExternFuncs,
    pub extern_arg_funcs: Vec<Ident>,
}

impl YapsConsumer {

    fn format_trait_ident(item: &ItemImpl) -> Ident {
        match utils::get_type_ident(&item.self_ty) {
            Some(i) => format_ident!("{}ExternFuncs", i),
            None => Ident::new("YapsExternFuncs", item.span())
        }
    }

    fn format_handle(func: &Ident) -> Ident {
        format_ident!("{}_handle", func)
    }

    fn generate_extern_trait(&self) -> ItemTrait {
        let funcs = self.extern_funcs.funcs.iter()
            .map(|func| -> TraitItemFn {
                let ident = &func.ident;
                let args = func.args.to_inputs();
                let ret_type = &func.ret_type;

                parse_quote! {
                    fn #ident(&self, #args) -> #YapsResult<#ret_type>;
                }
            });
        let ident = &self.extern_trait_ident;

        parse_quote! {
            trait #ident {
                #( #funcs )*
            }
        }
    }

    fn generate_extern_trait_impl(&self, for_type: &Ident) -> ItemImpl {
        let extern_trait_ident = &self.extern_trait_ident;
        let func_defs = self.extern_funcs.funcs.iter()
            .map(|func| -> ImplItemFn {
                let ident = &func.ident;
                let args = func.args.to_inputs();
                let ret_type = &func.ret_type;
                let handle_field = Self::format_handle(ident);
                let arg_idents_tuple = utils::punctuated_into_tuple(func.args.to_idents());

                parse_quote! {
                    fn #ident(&self, #args) -> #YapsResult<#ret_type> {
                        let data = self.serde.serialize(#arg_idents_tuple)?;
                        let result = self.#handle_field.call(data)?;
                        self.serde.deserialize(result)
                    }
                }
            });
        parse_quote! {
            impl<Data, Serde: #SerializerDeserializer<Data>> #extern_trait_ident for #for_type<Data, Serde> {
                #( #func_defs )*
            }
        }
    }

    fn generate_consumer_impl(&self, for_type: &Ident) -> ItemImpl {
        let funcs_init = self.extern_funcs.funcs.iter()
            .map(|func| -> Stmt {
                let id_str = LitStr::new(&func.id, func.ident.span());
                let handle_field = Self::format_handle(&func.ident);

                parse_quote! {
                    if let Ok(func) = provider.get_func(&#id_str.into()) {
                        self.0.borrow_mut().#handle_field = func;
                    }
                }
            });
    
        parse_quote! {
            impl<Data, Serde: #SerializerDeserializer<Data>> #FuncConsumer<Data> for #for_type<Data, Serde> {
            
                fn connect(&mut self, provider: &dyn #FuncProvider<Data>) -> #YapsResult<()> {
                    #( #funcs_init )*
                    Ok(())
                }
            
            }
        }
    }

    pub fn generate_handle_fields(&self) -> TokenStream {
        let handle_fields = self.extern_funcs.funcs.iter()
            .map(|func| {
                let handle_ident = Self::format_handle(&func.ident);
                quote! {
                    #handle_ident: #FunctionHandle<Data>
                }
            });
        quote! {
            #( #handle_fields, )*
        }
    }

    pub fn generate_handle_fields_init(&self) -> Punctuated<FieldValue, Token![,]> {
        let funcs_init = self.extern_funcs.funcs.iter()
            .map(|func| -> FieldValue {
                let handle_ident = Self::format_handle(&func.ident);
                let func_str = utils::ident_to_str(&func.ident);
                parse_quote! {
                    #handle_ident: #FunctionHandle::new(|_| {
                        Err(#YapsError::FunctionNotInitialized(#func_str.into()))
                    })
                }
            });
        parse_quote! {
            #( #funcs_init, )*
        }
    }

    pub fn process(item: &mut ItemImpl) -> Self {
        let extern_trait_ident = Self::format_trait_ident(item);
        let extern_funcs = ExternFuncs::process(item);
        let extern_arg_funcs = ExternArgs::process(item, extern_trait_ident.clone()).funcs;

        YapsConsumer {
            extern_trait_ident,
            extern_funcs,
            extern_arg_funcs,
        }
    }

    pub fn generate_code(&self, wrapper: &Ident, wrapper_ref: &Ident) -> TokenStream {
        let extern_trait = self.generate_extern_trait();
        let extern_trait_impl = self.generate_extern_trait_impl(wrapper);
        let consumer_impl = self.generate_consumer_impl(wrapper_ref);

        quote! {
            #extern_trait
            #extern_trait_impl
            #consumer_impl
        }
    }

}

