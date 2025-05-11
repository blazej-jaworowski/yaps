#![warn(missing_debug_implementations)]

pub use serde::{Serialize, de::DeserializeOwned as Deserialize};

mod json;
pub use json::{JsonCodec, JsonData};
