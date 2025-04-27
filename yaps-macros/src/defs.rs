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
            pub(crate) const $ident: crate::defs::LazyTokens =
                crate::defs::LazyTokens::new(|| ::quote::quote!{ $($tt)* });
        )*
    }
}

define_const_token_streams! {

    Vec = { ::std::vec::Vec };
    Box = { ::std::boxed::Box };
    Arc = { ::std::sync::Arc };
    Weak = { ::std::sync::Weak };
    OnceCell = { ::tokio::sync::OnceCell };
    async_trait = { ::yaps_core::async_trait::async_trait };

    Result = { ::yaps_core::Result };
    Error = { ::yaps_core::Error };

    Codec = { ::yaps_core::codec::Codec };
    EncodeFor = { ::yaps_core::codec::EncodeFor };
    DecodeFor = { ::yaps_core::codec::DecodeFor };

    FuncProvider = { ::yaps_core::FuncProvider };
    FuncConsumer = { ::yaps_core::FuncConsumer };
    FuncHandle = { ::yaps_core::FuncHandle };
    FuncMetadata = { ::yaps_core::FuncMetadata };

    ActorHandle = { ::yaps_core::actor_handle::ActorHandle };
    AsyncResult = { ::yaps_core::actor_handle::AsyncResult };

    YapsData = { ::yaps_core::YapsData };

}
