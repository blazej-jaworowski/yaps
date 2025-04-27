pub use crate::{Deserialize, Serialize};
use std::io::Cursor;
use yaps_core::{
    Error, Result,
    codec::{Codec, DecodeFor, EncodeFor},
};

#[derive(Debug, Clone, Default)]
pub struct JsonCodec;

impl Codec for JsonCodec {
    type Data = Vec<u8>;
}

impl<S: Serialize> EncodeFor<JsonCodec, S> for JsonCodec {
    fn encode(_codec: &JsonCodec, obj: S) -> Result<Vec<u8>> {
        let s = serde_json::to_string(&obj).map_err(|e| Error::Encode(e.to_string()))?;
        Ok(s.into())
    }
}

impl<D: Deserialize> DecodeFor<JsonCodec, D> for JsonCodec {
    fn decode(_codec: &JsonCodec, data: Vec<u8>) -> Result<D> {
        let c = Cursor::new(data);
        serde_json::from_reader(c).map_err(|e: serde_json::Error| Error::Decode(e.to_string()))
    }
}
