use crate::Result;

use std::rc::Rc;


pub type FunctionId = String;

#[derive(Clone)]
pub struct FunctionHandle<Data> {
    func: Rc<dyn Fn(Data) -> Result<Data>>,
}

impl<Data> FunctionHandle<Data> {

    pub fn call(&self, args: Data) -> Result<Data> {
        (*self.func)(args)
    }

    pub fn new<F: Fn(Data) -> Result<Data> + 'static>(func: F) -> Self {
        Self { func: Rc::new(func) }
    }

}

pub trait FuncProvider<Data> {

    fn provided_funcs(&self) -> Result<Vec<FunctionId>>;
    fn get_func(&self, id: &FunctionId) -> Result<FunctionHandle<Data>>;

}

pub trait FuncConsumer<Data> {

    fn connect(&mut self, provider: &dyn FuncProvider<Data>) -> Result<()>;

}
