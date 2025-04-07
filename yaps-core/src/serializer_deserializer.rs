use crate::{
    Result,
    YapsData,
    serde::{Serialize, de::DeserializeOwned},
};


pub trait SerializerDeserializer<Data: YapsData>: Send + Sync {

    fn serialize<S: Serialize>(&self, obj: S) -> Result<Data>;
    fn deserialize<D: DeserializeOwned>(&self, data: Data) -> Result<D>;

}
