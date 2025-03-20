use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, ImplItem, ImplItemFn, LitStr, Ident, FnArg, Pat, Type};

fn plugin_funcs(impl_block: &ItemImpl) -> impl Iterator<Item = &ImplItemFn> {
    impl_block.items.iter()
        .filter_map(|impl_item| {
            match impl_item {
                ImplItem::Fn(f) => Some(f),
                _ => None,
            }
        })
        .filter(|func| {
            func.attrs.iter().any(|attr| attr.path().is_ident("plugin_func"))
        })
}

fn make_serde_func(func: &ImplItemFn, serde_func: &Ident, data_type: &Type) -> proc_macro2::TokenStream {
    let func_ident = &func.sig.ident;

    let args = func.sig.inputs.iter()
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

    let var_ident = args.clone()
        .map(|(arg_ident, _)| {
            arg_ident
        });

    let var_ident_clone = var_ident.clone();

    let var_types = args.clone()
        .map(|(_, arg_type)| {
            arg_type
        });

    quote! {
        fn #serde_func(
            &self,
            args: #data_type,
        ) -> Result<#data_type> {
            let ( #( #var_ident ),* ): ( #( #var_types ),* ) = self.deserialize(args)?;

            let result = self.#func_ident( #( #var_ident_clone ),* );

            let serialized_result = self.serialize(result)?;

            Ok(serialized_result)
        }
    }
}

#[proc_macro_attribute]
pub fn plugin_connector(attr: TokenStream, item: TokenStream) -> TokenStream {
    let data_type = parse_macro_input!(attr as Type);

    let impl_block = parse_macro_input!(item as ItemImpl);

    let self_ty = &impl_block.self_ty;

    let mut provided_funcs = Vec::new();
    let mut serde_funcs = Vec::new();
    let mut match_arms = Vec::new();

    for func in plugin_funcs(&impl_block) {
        let func_ident = &func.sig.ident;

        let func_name = func_ident.to_string();
        let name_str = LitStr::new(&func_name, func_ident.span());

        provided_funcs.push(quote! { #name_str.into() });

        let serde_func_name = "serde_".to_string() + &func_name;
        let serde_func = Ident::new(&serde_func_name, func_ident.span());

        serde_funcs.push(make_serde_func(func, &serde_func, &data_type));

        match_arms.push(quote! {
            #name_str => {
                Ok(std::rc::Rc::new(|args: #data_type| -> Result<#data_type> {
                    self.#serde_func(args)
                }))
            }
        });
    }

    let generated = quote! {
        #impl_block

        const _: () = {
            use ::yaps_core::plugin_connector::WithSerde;
            fn _check<T: WithSerde<#data_type>>() {};
            _check::<#self_ty>;
        };

        impl #self_ty {
            #( #serde_funcs )*
        }

        impl ::yaps_core::PluginConnector<#data_type> for #self_ty {
            fn provided_funcs(&self) -> Vec<::yaps_core::FunctionId> {
                vec![ #( #provided_funcs ),* ]
            }

            fn get_func(&self, id: &::yaps_core::FunctionId) -> Result<::yaps_core::FunctionHandle<#data_type>> {
                match id.as_str() {
                    #( #match_arms ),*
                    _ => Err(::yaps_core::Error::FunctionNotFound(id.into()))
                }
            }
        }
    };

    generated.into()
}

#[proc_macro_attribute]
pub fn plugin_func(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
