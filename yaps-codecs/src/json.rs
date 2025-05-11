pub use crate::{Deserialize, Serialize};
use std::io::Cursor;
use yaps_core::{
    Error, Result, YapsData,
    codec::{Codec, DecodeFor, EncodeFor},
};

#[derive(Debug, Clone, Default)]
pub struct JsonCodec;

#[derive(Debug)]
pub struct JsonData(Vec<u8>);

impl YapsData for JsonData {}

impl Codec for JsonCodec {
    type Data = JsonData;
}

impl<S: Serialize> EncodeFor<JsonCodec, S> for JsonCodec {
    fn encode(_codec: &JsonCodec, obj: S) -> Result<JsonData> {
        let s = serde_json::to_string(&obj).map_err(|e| Error::Encode(e.to_string()))?;
        Ok(JsonData(s.into()))
    }
}

impl<D: Deserialize> DecodeFor<JsonCodec, D> for JsonCodec {
    fn decode(_codec: &JsonCodec, data: JsonData) -> Result<D> {
        let c = Cursor::new(data.0);
        serde_json::from_reader(c).map_err(|e: serde_json::Error| Error::Decode(e.to_string()))
    }
}
