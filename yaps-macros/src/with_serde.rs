use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    ItemStruct, Ident, Generics, Type,
    parse::{Parse, ParseStream},
    Result,
};
use darling::FromMeta;
use crate::utils::parse_type;


#[derive(Debug, FromMeta)]
pub struct WithSerdeArgs {
    #[darling(rename = "data", and_then = "parse_type")]
    data_type: Type,
    #[darling(and_then = "parse_type")]
    serde_type: Type,
}

pub struct WithSerdeInfo {
    struct_block: ItemStruct,
    struct_ident: Ident,
    generics: Generics,
}

impl WithSerdeInfo {

    pub fn generate(&self, args: WithSerdeArgs) -> TokenStream {
        let struct_block = self.struct_block.clone();
        let generics = self.generics.clone();
        let struct_ident = self.struct_ident.clone();
        let data_type = args.data_type.clone();
        let serde_type = args.serde_type;

        quote! {
            #struct_block

            impl #generics WithSerde<#data_type> for #struct_ident #generics {

                fn serde(&self) -> impl SerializerDeserializer<#data_type> {
                    #serde_type
                }

            }
        }
    }

}

impl Parse for WithSerdeInfo {

    fn parse(input: ParseStream) -> Result<Self> {
        let struct_block: ItemStruct = input.parse()?;

        Ok(WithSerdeInfo {
            struct_ident: struct_block.ident.clone(),
            generics: struct_block.generics.clone(),
            struct_block,
        })
    }

}
