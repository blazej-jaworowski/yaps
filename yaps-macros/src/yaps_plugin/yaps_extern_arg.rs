use syn::{
    parse_quote,
    Ident,
    ItemImpl,
    ImplItemFn,
    FnArg,
    Type,
    visit_mut::VisitMut,
};


#[derive(Debug)]
pub struct ExternArgs {
    extern_trait: Ident,
    pub funcs: Vec<Ident>,
}

impl VisitMut for ExternArgs {

    fn visit_impl_item_fn_mut(&mut self, item: &mut ImplItemFn) {
        // First arg should be &self
        if !matches!(item.sig.inputs.get(0), Some(FnArg::Receiver(_))) {
            return;
        };

        let second_arg = match item.sig.inputs.get_mut(1) {
            Some(FnArg::Typed(arg)) => arg,
            _ => return,
        };

        // Second arg should be of type YapsExtern
        if !matches!(&*second_arg.ty, Type::Path(p) if p.path.is_ident("YapsExtern")) {
            return;
        }

        let extern_trait = self.extern_trait.clone();

        second_arg.ty = parse_quote! {
            &impl #extern_trait
        };

        self.funcs.push(item.sig.ident.clone());
    }

}

impl ExternArgs {

    pub fn process(item: &mut ItemImpl, extern_trait: Ident) -> Self {
        let mut extern_funcs = ExternArgs { extern_trait, funcs: Vec::new() };
        extern_funcs.visit_item_impl_mut(item);
        extern_funcs
    }

}
