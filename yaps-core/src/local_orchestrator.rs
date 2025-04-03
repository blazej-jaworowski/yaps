use crate::{Result, Error};
use crate::{
    FunctionId, FunctionHandle, 
    FuncProvider, FuncConsumer,
};

use std::{
    rc::Rc,
    cell::RefCell,
};


pub struct LocalOrchestrator<Data> {

    providers: Vec<Rc<RefCell<dyn FuncProvider<Data>>>>,
    consumers: Vec<Rc<RefCell<dyn FuncConsumer<Data>>>>,

}

impl<Data> FuncProvider<Data> for LocalOrchestrator<Data> {
    
    fn provided_funcs(&self) -> Result<Vec<FunctionId>> {
        Ok(self.providers.iter()
            .filter_map(|p| p.borrow().provided_funcs().ok())
            .flatten()
            .collect())
    }

    fn get_func(&self, id: &FunctionId) -> Result<FunctionHandle<Data>> {
        let providers: Vec<_> = self.providers.iter()
            .filter(|p| p.borrow().provided_funcs()
                .is_ok_and(|p| p.contains(id)))
            .collect();

        if providers.is_empty() {
            return Err(Error::FunctionNotFound(id.into()));
        }

        // TODO: Maybe handle a case where there are multiple plugins

        providers[0].borrow().get_func(id)
    }

}

impl<Data> FuncConsumer<Data> for LocalOrchestrator<Data> {

    fn connect(&mut self, provider: &dyn FuncProvider<Data>) -> Result<()> {
        self.consumers.iter_mut()
            .for_each(|c| {
                c.borrow_mut().connect(provider).expect("TODO: handle later")
            });
        Ok(())
    }

}

impl<Data> LocalOrchestrator<Data> {

    pub fn new() -> Self {
        Self { providers: Vec::new(), consumers: Vec::new() }
    }

    pub fn add_provider(&mut self, provider: impl FuncProvider<Data> + 'static) -> Result<()> {
        self.connect(&provider)?;

        self.providers.push(Rc::new(RefCell::new(provider)));
        Ok(())
    }

    pub fn add_consumer(&mut self, mut consumer: impl FuncConsumer<Data> + 'static) -> Result<()> {
        consumer.connect(self)?;

        self.consumers.push(Rc::new(RefCell::new(consumer)));
        Ok(())
    }

    pub fn add_consumer_provider(&mut self, mut cp: impl FuncProvider<Data> + FuncConsumer<Data> + 'static) -> Result<()> {
        self.connect(&cp)?;
        cp.connect(self)?;

        let cp = Rc::new(RefCell::new(cp));

        self.providers.push(cp.clone());
        self.consumers.push(cp);
        Ok(())
    }

}
