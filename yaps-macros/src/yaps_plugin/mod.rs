mod yaps_extern_func;
mod yaps_init;

use proc_macro2::{TokenStream, Span};
use quote::quote;
use syn::{
    ItemImpl, ImplItem,
    LitStr, Ident, Type,
    Signature,
    Generics,
    Result,
    parse::{Parse, ParseStream},
};

use crate::utils::{func_args, parse_type};
use yaps_extern_func::YapsExternFuncs;
use yaps_init::YapsInitFunc;
use darling::FromMeta;

#[derive(Debug, FromMeta)]
pub struct YapsPluginArgs {
    #[darling(rename = "data", and_then = "parse_type")]
    data_type: Type,
}

pub struct YapsPluginInfo {
    impl_block: ItemImpl,

    init_func: YapsInitFunc,

    yaps_funcs: Vec<Signature>,
    wrapped_type: Type,
    generics: Generics,
}

/* Code parsing */

impl YapsPluginInfo {

    fn get_yaps_funcs(impl_block: &ItemImpl) -> Result<Vec<Signature>> {
        Ok(
            impl_block.items.iter()
                .filter_map(|impl_item| {
                    match impl_item {
                        ImplItem::Fn(f) => Some(f),
                        _ => None,
                    }
                })
                .filter(|func| {
                    func.attrs.iter().any(|attr| attr.path().is_ident("yaps_func"))
                })
                .map(|func| func.sig.clone())
                .collect()
        )
    }

}

impl Parse for YapsPluginInfo {

    fn parse(input: ParseStream) -> Result<Self> {
        let mut impl_block: ItemImpl = input.parse()?;

        let extern_funcs = YapsExternFuncs::process(&mut impl_block);
        let init_func = YapsInitFunc::process(&mut impl_block, extern_funcs.extern_funcs);

        let yaps_funcs = Self::get_yaps_funcs(&impl_block)?;
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
                            Ok(())
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

                #provided_funcs
                #get_func
            }
        }
    }

}
