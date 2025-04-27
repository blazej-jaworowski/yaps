use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ImplItemFn, ItemImpl, ItemStruct, LitStr, parse_quote};

use super::{yaps_extern::ExternFunc, yaps_plugin_macro::YapsPluginInfo};
use crate::{defs::*, utils};

fn wrapper_name(struct_name: &Ident) -> Ident {
    format_ident!("{}Wrapper", struct_name)
}

pub fn extern_field_name(func_name: &Ident) -> Ident {
    format_ident!("extern_{}", func_name)
}

pub(crate) fn generate_codec_export_bounds(info: &YapsPluginInfo) -> TokenStream {
    let in_types = info
        .export_funcs
        .iter()
        .map(|func| func.args.to_types())
        .map(utils::punctuated_into_tuple);
    let out_types = info.export_funcs.iter().map(|func| &func.ret_ty);

    let decode_for = DecodeFor;
    let encode_for = EncodeFor;

    quote! {
        #( + #decode_for<C, #in_types> )*
        #( + #encode_for<C, #out_types> )*
    }
}

pub(crate) fn generate_codec_extern_bounds(info: &YapsPluginInfo) -> TokenStream {
    let in_types = info
        .extern_funcs
        .iter()
        .map(|func| func.args.to_types())
        .map(utils::punctuated_into_tuple);
    let out_types = info.extern_funcs.iter().map(|func| &func.ret_ty);

    let decode_for = DecodeFor;
    let encode_for = EncodeFor;

    quote! {
        #( + #decode_for<C, #out_types> )*
        #( + #encode_for<C, #in_types> )*
    }
}

pub(crate) fn generate_wrapper_struct(info: &mut YapsPluginInfo) -> ItemStruct {
    let struct_ident = &info.struct_ident;
    let wrapper_ident = wrapper_name(&info.struct_ident);
    info.wrapper_ident = wrapper_ident.clone();

    let extern_fields = info
        .extern_funcs
        .iter()
        .map(|func| extern_field_name(&func.ident));

    let once_cell = OnceCell;
    let box_tok = Box;
    let func_handle = FuncHandle;

    parse_quote! {
        pub struct #wrapper_ident<D: #YapsData, C: #Codec<Data = D>> {
            pub inner: #Arc<#struct_ident>,
            codec: #Arc<C>,

            #( #extern_fields: #once_cell<#box_tok<dyn #func_handle<D>>>, )*
        }
    }
}

pub(crate) fn generate_wrapper_impl(info: &YapsPluginInfo) -> ItemImpl {
    let codec_export_bounds = generate_codec_export_bounds(info);
    let codec_extern_bounds = generate_codec_extern_bounds(info);

    let wrapper_ident = &info.wrapper_ident;
    let struct_ident = &info.struct_ident;

    let extern_fields = info
        .extern_funcs
        .iter()
        .map(|func| extern_field_name(&func.ident));

    let once_cell = OnceCell;

    parse_quote! {
        impl<C, D> #wrapper_ident<D, C>
        where
            D: #YapsData,
            C: #Codec<Data = D>
                #codec_export_bounds
                #codec_extern_bounds
                + 'static,
        {
            pub fn new(inner: #struct_ident, codec: C) -> #Arc<Self> {
                let new = #Arc::new(Self {
                    inner: #Arc::new(inner),
                    codec: #Arc::new(codec),

                    #( #extern_fields: #once_cell::new(), )*
                });

                let weak = #Arc::downgrade(&new);

                new.inner
                    .extern_funcs
                    .set(weak)
                    .expect("wrapping a plugin twice is not allowed");

                new
            }
        }
    }
}

fn generate_wrapper_extern_func_impl(func: &ExternFunc) -> ImplItemFn {
    let sig = &func.sig;
    let field_name = extern_field_name(&func.ident);
    let id_str = LitStr::new(&func.id, func.ident.span());

    parse_quote! {
        #sig {
            let func = self
                .#field_name
                .get()
                .ok_or(#Error::FunctionNotInitialized(#id_str.to_string()))?;

            func.call_with_codec(self.codec.as_ref(), (s,)).await
        }
    }
}

pub(crate) fn generate_wrapper_extern_funcs_impl(info: &YapsPluginInfo) -> ItemImpl {
    let codec_extern_bounds = generate_codec_extern_bounds(info);

    let extern_funcs_trait = &info.extern_funcs_trait;
    let wrapper_ident = &info.wrapper_ident;

    let extern_funcs_impls = info
        .extern_funcs
        .iter()
        .map(generate_wrapper_extern_func_impl);

    parse_quote! {
        #[#async_trait]
        impl<D, C> #extern_funcs_trait for #wrapper_ident<D, C>
        where
            D: #YapsData,
            C: #Codec<Data = D> #codec_extern_bounds,
        {
            #( #extern_funcs_impls )*
        }
    }
}
