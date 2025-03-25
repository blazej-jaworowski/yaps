use syn::{
    ImplItem, ItemImpl, ImplItemFn,
    Ident, Type, Signature,
    FnArg, Pat,
    punctuated::Punctuated,
    parse::{Parse, ParseStream},
    Token,
    Result,
};
use std::collections::HashMap;

fn impl_funcs(impl_block: &ItemImpl) -> impl Iterator<Item = &ImplItemFn> {
    impl_block.items.iter()
        .filter_map(|impl_item| {
            match impl_item {
                ImplItem::Fn(f) => Some(f),
                _ => None,
            }
        })
}

pub fn attr_funcs(impl_block: &ItemImpl, attr_name: String) -> impl Iterator<Item = &ImplItemFn> {
    impl_funcs(impl_block).filter(move |func| {
        func.attrs.iter().any(|attr| attr.path().is_ident(&attr_name))
    })
}

pub fn func_args(sig: &Signature) -> Vec<(Ident, Type)> {
    sig.inputs.iter()
        .filter_map(|input| {
            match input {
                FnArg::Typed(t) => Some(t),
                _ => None,
            }
        })
        .filter_map(|pat_type| {
            match &*pat_type.pat {
                Pat::Ident(pat_ident) => {
                    let arg_ident = pat_ident.ident.clone();
                    let arg_type = (*pat_type.ty).clone();
                    Some((arg_ident, arg_type))
                },
                _ => None,
            }
        })
        .collect()
}

struct MacroField<T> {
    key: Ident,
    #[allow(dead_code)]
    eq_token: Token![=],
    value: T,
}

impl<T> Parse for MacroField<T>
where
    T: Parse,
{

    fn parse(input: ParseStream) -> Result<Self> {
        Ok(MacroField {
            key: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }

}

// TODO: For now all arguments must have the same type, maybe change this in the future
pub struct MacroArgs<T>(HashMap<String, T>);

impl<T> MacroArgs<T> {

    pub fn pop_field(&mut self, field: &str) -> Option<T> {
        self.0.remove(field)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

}

impl<T> Parse for MacroArgs<T>
where
    T: Parse,
{

    fn parse(input: ParseStream) -> Result<Self> {
        let fields = Punctuated::<MacroField<T>, Token![,]>::parse_terminated(input)?;

        let fields: HashMap<String, T> = fields.into_iter()
            .map(|field| {
                (field.key.to_string(), field.value)
            }).collect();

        Ok(MacroArgs(fields))
    }

}
