use crate::{Result, Error};
use crate::plugin_connector::{PluginConnector, FunctionId, FunctionHandle};

use std::collections::HashMap;

pub type PluginId = String;
pub type FunctionKey = (PluginId, FunctionId);

pub trait Orchestrator<'a, Data> {

    fn register_plugin(&mut self, id: PluginId, plugin: Box<dyn PluginConnector<'a, Data>>) -> Result<()>;
    fn get_func(&self, key: FunctionKey) -> Result<FunctionHandle<'a, Data>>;

}

#[derive(Default)]
pub struct LocalOrchestrator<'a, Data> {

    plugins: HashMap<PluginId, Box<dyn PluginConnector<'a, Data>>>,

}

impl<'a, Data> Orchestrator<'a, Data> for LocalOrchestrator<'a, Data> {

    fn register_plugin(&mut self, id: PluginId, plugin: Box<dyn PluginConnector<'a, Data>>) -> Result<()> {
        if self.plugins.contains_key(&id) {
            return Err(Error::PluginRegistered(id));
        }

        self.plugins.insert(id, plugin);

        Ok(())
    }

    fn get_func(&self, key: FunctionKey) -> Result<FunctionHandle<'a, Data>> {
        let (plugin_id, function_id) = key;

        let plugin = self.plugins.get(&plugin_id).ok_or(Error::PluginNotFound(plugin_id))?;
        
        plugin.get_func(&function_id)
    }

}

