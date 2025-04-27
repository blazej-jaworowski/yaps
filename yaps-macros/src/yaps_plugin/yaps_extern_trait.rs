use syn::{parse_quote, ImplItemFn, ItemImpl, ItemTrait, LitStr};

use crate::defs::*;

use super::{yaps_extern::ExternFunc, yaps_plugin_macro::YapsPluginInfo};

pub(crate) fn generate_extern_trait(info: &YapsPluginInfo) -> ItemTrait {
    let ident = &info.extern_funcs_trait;
    let trait_items = info.extern_funcs.iter().map(|func| &func.sig);

    parse_quote! {
        #[#async_trait]
        trait #ident: Send + Sync {
            #( #trait_items; )*
        }
    }
}

fn generate_extern_trait_inner_fn(func: &ExternFunc, info: &YapsPluginInfo) -> ImplItemFn {
    let sig = &func.sig;
    let ident = &func.ident;
    let args = &func.args.to_idents();

    let plugin_str = LitStr::new(&info.plugin_name, info.struct_ident.span());

    parse_quote! {
        pub #sig {
            let extern_funcs = self
                .extern_funcs
                .get()
                .ok_or(#Error::PluginNotInitialized(#plugin_str.to_string()))?
                .upgrade()
                .ok_or(#Error::PluginWrapperDropped(#plugin_str.to_string()))?;

            extern_funcs.#ident(#args).await
        }
    }
}

pub(crate) fn generate_extern_funcs_inner_impl(info: &YapsPluginInfo) -> ItemImpl {
    let (impl_generics, ty_generics, where_generics) = info.struct_generics.split_for_impl();
    let ty_ident = &info.struct_ident;

    let items = info
        .extern_funcs
        .iter()
        .map(|func| generate_extern_trait_inner_fn(func, info));

    parse_quote! {
        impl<#impl_generics> #ty_ident #ty_generics #where_generics {
            #( #items )*
        }
    }
}
