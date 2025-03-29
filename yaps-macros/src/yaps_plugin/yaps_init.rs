use proc_macro2::TokenStream;
use syn::{
    ImplItemFn, ItemImpl,
    Ident, Attribute, Stmt,
    visit_mut::VisitMut,
    parse,
};
use quote::quote;

use super::yaps_extern_func::YapsExternFunc;

#[derive(Default)]
pub struct YapsInitFunc {
    extern_funcs: Vec<YapsExternFunc>,
    pub init_func: Option<Ident>,
}

fn is_init_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("yaps_init")
}

impl YapsInitFunc {

    pub fn process(item: &mut ItemImpl, extern_funcs: Vec<YapsExternFunc>) -> Self {
        let mut yaps_init_func = YapsInitFunc {
            extern_funcs,
            ..Default::default()
        };
        yaps_init_func.visit_item_impl_mut(item);
        yaps_init_func
    }

    fn generate_extern_func_init(func: &YapsExternFunc) -> TokenStream {
        let handle_ident = func.handle_ident.clone();
        let func_name = func.func_name.clone();
        let plugin_name = func.plugin_name.clone();

        quote! {
            let func_key = (#plugin_name.into(), #func_name.into());
            *self.#handle_ident.borrow_mut() = orchestrator.get_func(func_key)?;
        }
    }

    pub fn generate_funcs_init_block(&self) -> Stmt {
        let func_inits = self.extern_funcs.iter()
            .map(|extern_func| {
                Self::generate_extern_func_init(extern_func)
            });

        parse(quote!{
            {
                #( #func_inits )*
            }
        }.into()).expect("Error in generate_extern_func_init")
    }

    fn process_init_func(&mut self, func: &mut ImplItemFn) {
        self.init_func = Some(func.sig.ident.clone());
        func.attrs.retain(|attr| !is_init_attr(attr));

        let funcs_init_block = self.generate_funcs_init_block();

        func.block.stmts.insert(0, funcs_init_block);
    }

}

impl VisitMut for YapsInitFunc {

    fn visit_impl_item_fn_mut(&mut self, func: &mut ImplItemFn) {
        if func.attrs.iter().any(is_init_attr) {
            self.process_init_func(func);
        }
    }

}
