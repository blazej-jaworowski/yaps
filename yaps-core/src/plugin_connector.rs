use crate::Result;

use crate::serde::{Serialize, de::DeserializeOwned};
use crate::Orchestrator;

pub type FunctionHandle<'a, Data> = Box<dyn Fn(Data) -> Result<Data> + 'a>;
pub type FunctionId = String;

pub trait PluginConnector<'a, Data> {

    fn init(&self, orchestrator: &dyn Orchestrator<'a, Data>) -> Result<()>;
    fn provided_funcs(&self) -> Vec<FunctionId>;
    fn get_func(&self, id: &FunctionId) -> Result<FunctionHandle<'a, Data>>;

}

pub trait SerializerDeserializer<Data> {

    fn serialize<S: Serialize>(&self, obj: S) -> Result<Data>;
    fn deserialize<D: DeserializeOwned>(&self, data: Data) -> Result<D>;

}

pub trait WithSerde<Data> {

    fn serde(&self) -> impl SerializerDeserializer<Data>;

}
