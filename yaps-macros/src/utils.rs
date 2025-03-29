use syn::{
    parse_str,
    Ident, Type, Signature,
    FnArg, Pat,
};


pub fn parse_type(s: String) -> darling::Result<Type> {
    Ok(parse_str(&s)?)
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
