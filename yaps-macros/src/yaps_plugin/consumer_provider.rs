use quote::quote;
use syn::{Arm, Expr, ItemImpl, LitStr, parse_quote};

use super::wrapper::{extern_field_name, generate_codec_export_bounds};
use crate::{defs::*, utils};

use super::{yaps_export::ExportFunc, yaps_extern::ExternFunc, yaps_plugin_macro::YapsPluginInfo};

fn generate_func_metadata(export_func: &ExportFunc) -> Expr {
    let id_str = LitStr::new(&export_func.id, export_func.ident.span());

    parse_quote! {
        #FuncMetadata {
            id: #id_str.to_string(),
        }
    }
}

fn generate_provider_match_arm(export_func: &ExportFunc) -> Arm {
    let ident = &export_func.ident;
    let id_str = LitStr::new(&export_func.id, export_func.ident.span());

    let arg_types = utils::punctuated_into_tuple(export_func.args.to_types());
    let arg_idents_tuple = utils::punctuated_into_tuple(export_func.args.to_idents());
    let arg_idents = export_func.args.to_idents();

    let ret_type = &export_func.ret_ty;

    let await_call = if export_func.is_async {
        quote! { .await }
    } else {
        quote! {}
    };

    parse_quote! {
        #id_str => {
            // TODO: handle the join handle
            let (handle, _) = #ActorHandle::spawn_with_codec(
                move |args| -> #AsyncResult<#ret_type> {
                    let inner = inner.clone();
                    #Box::pin(async move {
                        let inner = inner.upgrade().ok_or(#Error::HandlerInvalidated)?;

                        // This will change in the macro
                        let #arg_idents_tuple: #arg_types = args;
                        Ok(inner.#ident(#arg_idents) #await_call)
                    })
                },
                self.codec.clone(),
            )?;
            Ok(#Box::new(handle))
        }
    }
}

pub(crate) fn generate_provider_impl(info: &YapsPluginInfo) -> ItemImpl {
    let codec_export_bounds = generate_codec_export_bounds(info);
    let wrapper_ident = &info.wrapper_ident;

    let func_metadatas = info.export_funcs.iter().map(generate_func_metadata);
    let func_arms = info.export_funcs.iter().map(generate_provider_match_arm);

    parse_quote! {
        #[#async_trait]
        impl<D, C> #FuncProvider<D> for #wrapper_ident<D, C>
        where
            D: #YapsData,
            C: #Codec<Data = D> #codec_export_bounds + 'static,
        {
            async fn provided_funcs(&self) -> #Result<#Vec<#FuncMetadata>> {
                Ok(#Vec::from([ #( #func_metadatas ),* ]))
            }

            async fn get_func(&self, id: &str) -> #Result<#Box<dyn #FuncHandle<D>>> {
                let inner = #Arc::downgrade(&self.inner);
                match id {
                    #( #func_arms, )*
                    _ => Err(#Error::FunctionNotFound(id.to_string())),
                }
            }
        }
    }
}

fn generate_consumer_match_arm(extern_func: &ExternFunc) -> Arm {
    let id_str = LitStr::new(&extern_func.id, extern_func.ident.span());
    let extern_field = extern_field_name(&extern_func.ident);

    parse_quote! {
        #id_str => {
            let func_handle = provider.get_func(#id_str).await?;
            match self.#extern_field.set(func_handle) {
                Ok(()) => {}
                Err(_) => {} // TODO: function set already, maybe log this
            }
        }
    }
}

pub(crate) fn generate_consumer_impl(info: &YapsPluginInfo) -> ItemImpl {
    let extern_arms = info.extern_funcs.iter().map(generate_consumer_match_arm);
    let wrapper_ident = &info.wrapper_ident;

    parse_quote! {
        #[#async_trait]
        impl<D: #YapsData, C: #Codec<Data = D>> #FuncConsumer<D> for #wrapper_ident<D, C> {
            async fn connect(&self, provider: &dyn #FuncProvider<D>) -> #Result<()> {
                for func in provider.provided_funcs().await? {
                    match func.id.as_str() {
                        #( #extern_arms, )*
                        _ => continue,
                    }
                }

                Ok(())
            }
        }
    }
}
