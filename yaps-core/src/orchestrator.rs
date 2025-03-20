use crate::{Result, Error};
use crate::plugin_connector::{PluginConnector, FunctionId, FunctionHandle};

use std::collections::HashMap;

pub trait Orchestrator<'a, Data> {

    fn register_plugin<P: PluginConnector<Data> + 'a>(&mut self, plugin: P) -> Result<()>;

    fn get_func(&self, id: &FunctionId) -> Result<FunctionHandle<Data>>;

}

#[derive(Default)]
pub struct LocalOrchestrator<'a, Data> {

    plugins: Vec<Box<dyn PluginConnector<Data> + 'a>>,
    funcs_map: HashMap<FunctionId, usize>,

}

impl<'a, Data> Orchestrator<'a, Data> for LocalOrchestrator<'a, Data> {

    fn register_plugin<P: PluginConnector<Data> + 'a>(&mut self, plugin: P) -> Result<()> {
        let index = self.plugins.len();

        for func_id in plugin.provided_funcs() {
            self.funcs_map.insert(func_id, index);
        }

        self.plugins.push(Box::new(plugin));

        Ok(())
    }

    fn get_func(&self, id: &FunctionId) -> Result<FunctionHandle<Data>> {
        let plugin_index = *self.funcs_map.get(id)
            .ok_or_else(|| Error::FunctionNotFound(id.into()))?;

        
        let plugin = &self.plugins[plugin_index];
        plugin.get_func(id)
    }

}

