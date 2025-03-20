use crate::Result;

use std::rc::Rc;
use crate::serde::{Serialize, de::DeserializeOwned};

pub type FunctionHandle<'a, Data> = Rc<dyn Fn(Data) -> Result<Data> + 'a>;
pub type FunctionId = String;

pub trait PluginConnector<Data> {

    fn provided_funcs(&self) -> Vec<FunctionId>;
    fn get_func(&self, id: &FunctionId) -> Result<FunctionHandle<Data>>;

}

pub trait WithSerde<Data> {

    fn serialize<S: Serialize>(&self, obj: S) -> Result<Data>;
    fn deserialize<D: DeserializeOwned>(&self, data: Data) -> Result<D>;

}
