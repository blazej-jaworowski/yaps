use proc_macro2::{TokenStream, Span};
use quote::quote;
use syn::{
    ItemStruct, Ident, Type, Generics,
    parse::{Parse, ParseStream},
    Result, Error,
};
use crate::utils::MacroArgs;

pub struct WithSerdeArgs {
    data_type: Type,
    serde_type: Type,
}

impl Parse for WithSerdeArgs {

    fn parse(input: ParseStream) -> Result<Self> {
        let mut args: MacroArgs<Type> =  input.parse()?;

        let data_type = args.pop_field("data")
            .ok_or(Error::new(Span::call_site(), "'data' macro field is required"))?;

        let serde_type = args.pop_field("serde_type")
            .ok_or(Error::new(Span::call_site(), "'serde_type' macro field is required"))?;

        if !args.is_empty() {
            return Err(Error::new(Span::call_site(), "Unexpected macro args"));
        }

        Ok(WithSerdeArgs {
            data_type,
            serde_type
        })
    }

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
