use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_quote,
    punctuated::Punctuated,
    Token,
    Ident, Type,
    LitStr,
    Signature, Attribute,
    FnArg, Pat,
};


#[derive(Debug, Clone)]
pub struct FunctionArgs(pub Vec<(Ident, Type)>);

impl From<&Signature> for FunctionArgs {
    
    fn from(sig: &Signature) -> Self {
        let args_vec = sig.inputs.iter()
            .filter_map(|input| {
                match input {
                    FnArg::Typed(t) => Some(t),
                    _ => None,
                }
            })
            .filter_map(|pat_type| {
                match &*pat_type.pat {
                    // TODO: add support
                    //Pat::Wild(_) => { /* ... */ },
                    Pat::Ident(pat_ident) => {
                        let arg_ident = pat_ident.ident.clone();
                        let arg_type = (*pat_type.ty).clone();
                        Some((arg_ident, arg_type))
                    },
                    _ => None,
                }
            })
            .collect();
        Self(args_vec)
    }

}

impl From<Vec<(Ident, Type)>> for FunctionArgs {
    
    fn from(vec: Vec<(Ident, Type)>) -> Self {
        Self(Vec::from(vec))
    }

}

impl FunctionArgs {

    pub fn to_inputs(&self) -> Punctuated<FnArg, Token![,]> {
        let fn_args = self.0.iter()
            .map(|(ident, ty)| -> FnArg { parse_quote!{ #ident: #ty } });
        parse_quote!{ #( #fn_args ),* }
    }

    pub fn to_idents(&self) -> Punctuated<Ident, Token![,]> {
        let idents = self.0.iter().map(|(ident, _)| ident);
        parse_quote!{ #( #idents ),* }
    }

    pub fn to_types(&self) -> Punctuated<Type, Token![,]> {
        let types = self.0.iter().map(|(_, ty)| ty);
        parse_quote!{ #( #types ),* }
    }

}

pub fn punctuated_into_tuple<T: ToTokens>(mut p: Punctuated<T, Token![,]>) -> TokenStream {
    if !p.empty_or_trailing() {
        p.push_punct(parse_quote!{,});
    }

    quote!{ (#p) }
}

pub fn ident_to_str(ident: &Ident) -> LitStr {
    let s = ident.to_string();
    LitStr::new(&s, ident.span())
}

pub fn get_type_ident(ty: &Type) -> Option<&Ident> {
    match ty {
        Type::Path(p) => p.path.get_ident(),
        _ => None,
    }
}

pub fn get_attr<'a>(attributes: &'a Vec<Attribute>, ident: &str) -> Option<&'a Attribute> {
    attributes.iter().find(|attr| attr.path().is_ident(ident))
}

pub fn pop_attr(attributes: &mut Vec<Attribute>, ident: &str) -> Option<Attribute> {
    let attr = match get_attr(&attributes, ident) {
        Some(a) => a.clone(),
        None => return None,
    };

    attributes.retain(|attr| !attr.path().is_ident(ident));

    Some(attr)
}

macro_rules! format_ident {
    ($format:literal, $ident:expr) => {{
        let s = $ident.to_string();
        let s = format!($format, s);
        syn::Ident::new(&s, $ident.span())
    }}
}
