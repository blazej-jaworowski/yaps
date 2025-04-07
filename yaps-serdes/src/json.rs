use serde::{Serialize, de::DeserializeOwned};
use yaps_core::{
    Result, Error,
    serializer_deserializer::SerializerDeserializer,
};
use std::io::Cursor;

pub struct JsonSerde;

impl SerializerDeserializer<Vec<u8>> for JsonSerde {

    fn serialize<S: Serialize>(&self, obj: S) -> Result<Vec<u8>> {
        let s = serde_json::to_string(&obj).map_err(|e| {
            Error::SerializeError(e.to_string())
        })?;
        Ok(s.into())
    }

    fn deserialize<D: DeserializeOwned>(&self, data: Vec<u8>) -> Result<D> {
        let c = Cursor::new(data);
        serde_json::from_reader(c).map_err(|e: serde_json::Error| {
            Error::DeserializeError(e.to_string())
        })
    }

}
