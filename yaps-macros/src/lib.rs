use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, ImplItem, ImplItemFn, LitStr, Ident, FnArg, Pat, Type, Error};

fn yaps_funcs(impl_block: &ItemImpl) -> impl Iterator<Item = &ImplItemFn> {
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
}

#[proc_macro_attribute]
pub fn yaps_plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    if attr.is_empty() {
        let error = Error::new(Span::call_site(), "You need to provide serialized data type");
        return error.to_compile_error().into();
    }

    let impl_block = parse_macro_input!(item as ItemImpl);
    let data_type = parse_macro_input!(attr as Type);

    let self_ty = &impl_block.self_ty;

    let generics = &impl_block.generics;

    let mut provided_funcs = Vec::new();
    let mut match_arms = Vec::new();

    for func in yaps_funcs(&impl_block) {
        let func_ident = &func.sig.ident;

        let func_name = func_ident.to_string();
        let name_str = LitStr::new(&func_name, func_ident.span());

        provided_funcs.push(quote! { #name_str.into() });

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

        // TODO: A lot of cloning is happening here, maybe there is a better way
        let var_ident = args.clone()
            .map(|(arg_ident, _)| {
                arg_ident
            });

        let var_ident_clone = var_ident.clone();

        let var_types = args
            .map(|(_, arg_type)| {
                arg_type
            });

        match_arms.push(quote! {
            #name_str => {
                let wrapped = self.0.clone();
                Ok(Box::new(move |args: #data_type| -> Result<#data_type> {
                    let ( #( #var_ident ),* ): ( #( #var_types ),* ) = wrapped.deserialize(args)?;

                    let result = wrapped.#func_ident( #( #var_ident_clone ),* );

                    let serialized_result = wrapped.serialize(result)?;

                    Ok(serialized_result)
                }))
            }
        });
    }

    // TODO: Let the wrapper type name be set by the user in the future
    let mut wrapper_type = Ident::new("WrappedPlugin", Span::call_site());
    if let Type::Path(p) = *self_ty.clone() {

        if let Some(last_segment) = p.path.segments.last() {
            let last_segment = last_segment.ident.clone();
            wrapper_type = Ident::new(&(last_segment.to_string() + "Connector"), Span::call_site());
        }
    }

    let plugin_lifetime = quote! { 'plugin };

    // We need to add the plugin lifetime to the generic's if it's not already there
    let plugin_generics = if generics.lifetimes().any(|lifetime| {
        lifetime.lifetime.ident == "plugin"
    }) {
        quote! { #generics }
    } else {
        let params = generics.params.clone();
        quote!{ < 'plugin, #params >  }
    };

    let generated = quote! {
        // Leave original code as is
        #impl_block

        // Define a wrapper type
        struct #wrapper_type #generics(std::rc::Rc<#self_ty>);

        impl #generics #self_ty {

            // Helper
            fn wrap(self) -> #wrapper_type #generics {
                #wrapper_type::new(self)
            }

        }

        impl #generics #wrapper_type #generics {

            // Helper
            fn new(wrapped: #self_ty) -> #wrapper_type #generics {
                #wrapper_type(std::rc::Rc::new(wrapped))
            }

        }

        // We need the additional 'plugin lifetime here
        impl #plugin_generics ::yaps_core::PluginConnector<#plugin_lifetime, #data_type> for #wrapper_type #generics {

            fn provided_funcs(&self) -> Vec<::yaps_core::FunctionId> {
                vec![ #( #provided_funcs ),* ]
            }

            fn get_func(&self, id: &::yaps_core::FunctionId) -> Result<::yaps_core::FunctionHandle<#plugin_lifetime, #data_type>> {
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
pub fn yaps_func(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // This attribute is just a marker
    item
}
