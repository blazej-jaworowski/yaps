use proc_macro2::{TokenStream as TokenStream2, Span};
use quote::quote;
use syn::{
    ItemImpl,
    LitStr, Ident, Type,
    FnArg, Pat,
    Signature,
    Generics,
    Result, Error,
    parse::{Parse, ParseStream},
};

use crate::utils::{attr_funcs, MacroArgs};

pub struct YapsPluginArgs {
    data_type: Type,
}

// Arg parsing
impl Parse for YapsPluginArgs {

    fn parse(input: ParseStream) -> Result<Self> {
        let mut args: MacroArgs<Type> = input.parse()?;

        let data_type = args.pop_field("data")
            .ok_or(Error::new(Span::call_site(), "'data' macro field is required"))?;

        if !args.is_empty() {
            return Err(Error::new(Span::call_site(), "Unexpected macro args"));
        }

        Ok(YapsPluginArgs {
            data_type,
        })
    }

}

pub struct YapsPluginInfo {
    impl_block: ItemImpl,
    init_func: Option<Ident>,
    yaps_funcs: Vec<Signature>,
    wrapped_type: Type,
    generics: Generics,
}

/* Code parsing */

impl YapsPluginInfo {

    fn get_yaps_funcs(impl_block: &ItemImpl) -> Result<Vec<Signature>> {
        Ok(
            attr_funcs(impl_block, "yaps_func".into())
                .map(|func| func.sig.clone())
                .collect()
        )
    }

    fn get_init_func(impl_block: &ItemImpl) -> Result<Option<Ident>> {
        let init_funcs: Vec<Ident> = attr_funcs(impl_block, "yaps_plugin_init".into())
            .map(|func| {
                func.sig.ident.clone()
            })
            .collect();

        if init_funcs.len() > 1 {
            return Err(Error::new(Span::call_site(), "Only one init func allowed"));
        }

        Ok(init_funcs.first().cloned())
    }

}

impl Parse for YapsPluginInfo {

    fn parse(input: ParseStream) -> Result<Self> {
        let impl_block: ItemImpl = input.parse()?;

        let yaps_funcs = Self::get_yaps_funcs(&impl_block)?;
        let init_func = Self::get_init_func(&impl_block)?;
        let wrapped_type = *impl_block.self_ty.clone();
        let generics = impl_block.generics.clone();

        Ok(YapsPluginInfo {
            impl_block,
            init_func,
            yaps_funcs,
            wrapped_type,
            generics,
        })
    }

}

/* Code generation */

impl YapsPluginInfo {

    fn wrapper_type(&self) -> Ident {
        // TODO: The user should be able to set this

        if let Type::Path(p) = &self.wrapped_type {
            if let Some(last_segment) = p.path.segments.last() {
                let wrapper_name = format!("{}Connector", last_segment.ident);
                return Ident::new(&wrapper_name, Span::call_site());
            }
        }

        // Fallback name
        Ident::new("WrappedPlugin", Span::call_site())
    }

    fn generate_wrapper_type_definition(&self) -> TokenStream2 {
        let wrapper_type = self.wrapper_type();
        let generics = self.generics.clone();
        let wrapped_type = self.wrapped_type.clone();

        quote! {
            struct #wrapper_type #generics(std::rc::Rc<#wrapped_type>);
        }
    }

    fn generate_wrap_helper(&self) -> TokenStream2 {
        let generics = self.generics.clone();
        let wrapped_type = self.wrapped_type.clone();
        let wrapper_type = self.wrapper_type();

        quote! {
            impl #generics #wrapped_type {

                fn wrap(self) -> #wrapper_type #generics {
                    #wrapper_type::new(self)
                }

            }
        }
    }

    fn generate_new_helper(&self) -> TokenStream2 {
        let wrapped_type = self.wrapped_type.clone();
        let generics = self.generics.clone();
        let wrapper_type = self.wrapper_type();

        quote! {
            impl #generics #wrapper_type #generics {

                fn new(wrapped: #wrapped_type) -> #wrapper_type #generics {
                    #wrapper_type(std::rc::Rc::new(wrapped))
                }

            }
        }
    }

    fn generate_provided_funcs(&self) -> TokenStream2 {
        let funcs_names = self.yaps_funcs.iter()
            .map(|func| LitStr::new(&func.ident.to_string(), func.ident.span()));

        quote! {
            fn provided_funcs(&self) -> Vec<String> {
                vec![ #( #funcs_names.into() ),* ]
            }
        }
    }

    fn generate_init_func(&self) -> TokenStream2 {
        let init_body = if let Some(init_func) = &self.init_func {
            let init_func = init_func.clone();
            quote! {
                self.0.#init_func(orchestrator)
            }
        } else {
            quote! {
                Ok(())
            }
        };

        quote! {
            fn init(
                &self,
                orchestrator: &dyn Orchestrator<'plugin, Vec<u8>>
            ) -> Result<()> {
                #init_body
            }
        }
    }

    fn generate_match_arm(&self, args: &YapsPluginArgs, func: &Signature) -> TokenStream2 {
        let func_ident = func.ident.clone();
        let name_str = LitStr::new(&func.ident.to_string(), func.ident.span());
        let data_type = args.data_type.clone();

        let func_args = func.inputs.iter()
            .filter_map(|input| {
                match input {
                    FnArg::Typed(t) => Some(t),
                    _ => None,
                }
            })
            .filter_map(|pat_type| {
                match &*pat_type.pat {
                    Pat::Ident(pat_ident) => {
                        let arg_ident = &pat_ident.ident;
                        let arg_type = &pat_type.ty;
                        Some((arg_ident, arg_type))
                    },
                    _ => None,
                }
            });

        let var_ident = func_args.clone()
            .map(|(arg_ident, _)| {
                arg_ident
            });

        let var_ident_clone = var_ident.clone();

        let var_types = func_args
            .map(|(_, arg_type)| {
                arg_type
            });

        quote! {
            #name_str => {
                let wrapped = self.0.clone();
                Ok(Box::new(move |args: #data_type| -> Result<#data_type> {
                    let serde = wrapped.serde();
                    let ( #( #var_ident ),* ): ( #( #var_types ),* ) = serde.deserialize(args)?;
                    let result = wrapped.#func_ident( #( #var_ident_clone ),* );
                    let serialized_result = serde.serialize(result)?;
                    Ok(serialized_result)
                }))
            }
        }
    }

    fn generate_get_func(&self, args: &YapsPluginArgs) -> TokenStream2 {
        let data_type = args.data_type.clone();
        let match_arms = self.yaps_funcs.iter()
            .map(|func| {
                self.generate_match_arm(args, func)
            });

        quote! {
            fn get_func(
                &self, id: &::yaps_core::FunctionId
            ) -> Result<::yaps_core::FunctionHandle<'plugin, #data_type>> {
                match id.as_str() {
                    #( #match_arms ),*
                    _ => Err(::yaps_core::Error::FunctionNotFound(id.into()))
                }
            }
        }
    }

    pub fn generate(self, args: YapsPluginArgs) -> TokenStream2 {
        let generics = self.generics.clone();
        let impl_block = self.impl_block.clone();
        let data_type = args.data_type.clone();

        let plugin_generics = if self.generics.lifetimes().any(|lifetime| {
            lifetime.lifetime.ident == "plugin"
        }) {
            quote! { #generics }
        } else {
            let params = self.generics.params.clone();
            quote!{ < 'plugin, #params >  }
        };

        let wrapper_type = self.wrapper_type();
        let wrapper_type_definition = self.generate_wrapper_type_definition();
        let wrap_helper = self.generate_wrap_helper();
        let new_helper = self.generate_new_helper();

        let init_func = self.generate_init_func();
        let provided_funcs = self.generate_provided_funcs();
        let get_func = self.generate_get_func(&args);

        quote! {
            // Leave original code as is
            #impl_block

            #wrapper_type_definition
            #wrap_helper
            #new_helper

            impl #plugin_generics ::yaps_core::PluginConnector<'plugin, #data_type>
                for #wrapper_type #generics {
                #init_func
                #provided_funcs
                #get_func
            }
        }
    }

}
