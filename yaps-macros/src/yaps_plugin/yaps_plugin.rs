use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote,
    Token,
    Type, Ident,
    ItemImpl, ItemStruct,
    FieldValue,
    punctuated::Punctuated,
    spanned::Spanned,
};

use crate::utils;

use super::yaps_consumer::YapsConsumer;
use super::yaps_provider::YapsProvider;
use super::defs::*;

fn wrapper_name(inner_type: &Type) -> Ident {
    match utils::get_type_ident(inner_type) {
        Some(i) => format_ident!("{}Wrapper", i),
        None => Ident::new("YapsWrapper", inner_type.span()),
    }
}

fn wrapper_ref_name(inner_type: &Type) -> Ident {
    match utils::get_type_ident(inner_type) {
        Some(i) => format_ident!("{}WrapperRef", i),
        None => Ident::new("YapsWrapperRef", inner_type.span()),
    }
}

fn generate_wrapper(inner_type: &Type, extra_fields: &TokenStream) -> ItemStruct {
    let wrapper_name = wrapper_name(inner_type);

    parse_quote! {
        struct #wrapper_name<Data, Serde: #SerializerDeserializer<Data>> {
            _marker: ::std::marker::PhantomData<Data>,
            inner: #inner_type,
            serde: Serde,
        
            #extra_fields
        }
    }
}

fn generate_wrapper_wrap(inner_type: &Type, extra_field_init: &Punctuated<FieldValue, Token![,]>) -> ItemImpl {
    let wrapper_name = wrapper_name(inner_type);
    let wrapper_ref_name = wrapper_ref_name(inner_type);

    parse_quote! {
        impl<Data, Serde: #SerializerDeserializer<Data>> #wrapper_name<Data, Serde> {
        
            fn wrap(inner: #inner_type, serde: Serde) -> #wrapper_ref_name<Data, Serde> {
                let wrapper = #wrapper_name {
                    _marker: std::marker::PhantomData,
                    inner,
                    serde,
        
                    #extra_field_init
                };
                #wrapper_ref_name(#Rc::new(#RefCell::new(wrapper)))
            }
        
        }
    }
}

pub fn process_yaps_plugin(item: &mut ItemImpl) -> TokenStream {
    let consumer = YapsConsumer::process(item);
    let provider = YapsProvider::process(item);

    let inner_type = &*item.self_ty;

    let wrapper_ident = wrapper_name(inner_type);
    let wrapper_ref_ident = wrapper_ref_name(inner_type);

    let extra_wrapper_fields = consumer.generate_handle_fields();
    let extra_wrapper_field_init = consumer.generate_handle_fields_init();
    let extern_arg_funcs = &consumer.extern_arg_funcs;

    let wrapper = generate_wrapper(&inner_type, &extra_wrapper_fields);
    let wrapper_wrap = generate_wrapper_wrap(&inner_type, &extra_wrapper_field_init);

    let consumer_code = consumer.generate_code(&wrapper_ident, &wrapper_ref_ident);
    let provider_code = provider.generate_code(&wrapper_ref_ident, extern_arg_funcs);

    quote! {
        #item

        struct #wrapper_ref_ident<Data, Serde: #SerializerDeserializer<Data>>(#Rc<#RefCell<#wrapper_ident<Data, Serde>>>);

        #wrapper
        #wrapper_wrap

        #consumer_code
        #provider_code
    }
}
