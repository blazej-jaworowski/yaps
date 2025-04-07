#![allow(non_upper_case_globals)]
#![allow(clippy::declare_interior_mutable_const)]

use proc_macro2::TokenStream;
use std::cell::LazyCell;
use quote::ToTokens;

pub(crate) struct LazyTokens(LazyCell<TokenStream>);

impl LazyTokens {

    const fn new(func: fn() -> TokenStream) -> Self {
        LazyTokens(LazyCell::new(func))
    }

}

impl ToTokens for LazyTokens {
    
    fn to_tokens(&self, stream: &mut TokenStream) {
        let content = self.0.clone();
        content.to_tokens(stream)
    }

}

macro_rules! define_const_token_streams {
    ($( $ident:ident = { $($tt:tt)* } ;)*) => {
        $(
            pub(crate) const $ident: crate::yaps_plugin::defs::LazyTokens =
                crate::yaps_plugin::defs::LazyTokens::new(|| ::quote::quote!{ $($tt)* });
        )*
    }
}

define_const_token_streams! {

    Arc = { ::std::sync::Arc };
    RwLock = { ::tokio::sync::RwLock };
    async_trait = { ::yaps_core::async_trait::async_trait };

    SerializerDeserializer = { ::yaps_core::serializer_deserializer::SerializerDeserializer };
    FuncProvider = { ::yaps_core::consumer_provider::FuncProvider };
    FuncConsumer = { ::yaps_core::consumer_provider::FuncConsumer };
    FunctionId = { ::yaps_core::consumer_provider::FunctionId };
    FunctionHandle = { ::yaps_core::consumer_provider::FunctionHandle };
    YapsError = { ::yaps_core::Error };
    YapsResult = { ::yaps_core::Result };

    YapsData = { ::yaps_core::YapsData };

}
