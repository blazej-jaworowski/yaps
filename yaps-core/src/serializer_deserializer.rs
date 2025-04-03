use crate::{
    Result,
    serde::{Serialize, de::DeserializeOwned},
};


pub trait SerializerDeserializer<Data> {

    fn serialize<S: Serialize>(&self, obj: S) -> Result<Data>;
    fn deserialize<D: DeserializeOwned>(&self, data: Data) -> Result<D>;

}
