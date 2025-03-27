use proc_macro2::{TokenStream, Span};
use quote::quote;
use syn::{
    ItemImpl,
    LitStr, Ident, Type,
    Signature,
    Generics,
    Result, Error,
    parse::{Parse, ParseStream},
    visit_mut::VisitMut,
};

use crate::utils::{attr_funcs, func_args, MacroArgs};
use super::{
    yaps_extern_func::YapsExternFuncs,
    yaps_init::YapsInitFunc,
};

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

    extern_funcs: YapsExternFuncs,
    init_func: YapsInitFunc,

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

}

impl Parse for YapsPluginInfo {

    fn parse(input: ParseStream) -> Result<Self> {
        let mut impl_block: ItemImpl = input.parse()?;

        let mut extern_funcs = YapsExternFuncs::default();
        extern_funcs.visit_item_impl_mut(&mut impl_block);

        let mut init_func = YapsInitFunc::new(extern_funcs.extern_funcs.clone());
        init_func.visit_item_impl_mut(&mut impl_block);

        let yaps_funcs = Self::get_yaps_funcs(&impl_block)?;
        let wrapped_type = *impl_block.self_ty.clone();
        let generics = impl_block.generics.clone();

        Ok(YapsPluginInfo {
            impl_block,

            extern_funcs,
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

    fn generate_provided_funcs(&self) -> TokenStream {
        let funcs_names = self.yaps_funcs.iter()
            .map(|func| LitStr::new(&func.ident.to_string(), func.ident.span()));

        quote! {
            fn provided_funcs(&self) -> Vec<String> {
                vec![ #( #funcs_names.into() ),* ]
            }
        }
    }

    fn generate_match_arm(&self, args: &YapsPluginArgs, func: &Signature) -> TokenStream {
        let func_ident = func.ident.clone();
        let name_str = LitStr::new(&func.ident.to_string(), func.ident.span());
        let data_type = args.data_type.clone();

        let func_args = func_args(func);

        let var_ident = func_args.iter()
            .map(|(arg_ident, _)| {
                arg_ident
            });

        let var_ident_clone = var_ident.clone();

        let var_types = func_args.iter()
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

    fn generate_get_func(&self, args: &YapsPluginArgs) -> TokenStream {
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

    pub fn generate(self, args: YapsPluginArgs) -> TokenStream {
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

        let wrapped_type = self.wrapped_type.clone();
        let wrapper_type = self.wrapper_type();

        let (init_func_ident, init_func_definition) = match &self.init_func.init_func {
            Some(i) => (
                i.clone(),
                quote! {},
            ),
            None => {
                let init_body = self.init_func.generate_funcs_init_block();
                (
                    Ident::new("init", Span::call_site()),
                    quote! {
                        fn init(
                            &self,
                            orchestrator: &dyn Orchestrator<'plugin, Vec<u8>>,
                        ) -> Result<()> {
                            #init_body
                        }
                    }
                )
            },
        };
        let provided_funcs = self.generate_provided_funcs();
        let get_func = self.generate_get_func(&args);

        quote! {
            // Leave original code as is
            #impl_block

            // Wrapper type definition
            struct #wrapper_type #generics(std::rc::Rc<#wrapped_type>);

            // Wrap helper
            impl #generics #wrapped_type {
                fn wrap(self) -> #wrapper_type #generics {
                    #wrapper_type::new(self)
                }

                #init_func_definition
            }

            // New helper
            impl #generics #wrapper_type #generics {
                fn new(wrapped: #wrapped_type) -> #wrapper_type #generics {
                    #wrapper_type(std::rc::Rc::new(wrapped))
                }
            }

            impl #plugin_generics ::yaps_core::PluginConnector<'plugin, #data_type>
                for #wrapper_type #generics {

                fn init(
                    &self,
                    orchestrator: &dyn Orchestrator<'plugin, Vec<u8>>,
                ) -> Result<()> {
                    self.0.#init_func_ident(orchestrator)
                }

                #init_func_ident;
                #provided_funcs
                #get_func
            }
        }
    }

}
