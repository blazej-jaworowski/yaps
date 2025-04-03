use crate::FunctionId;
use crate::serde::{Serialize, Deserialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq, Eq, Debug, thiserror::Error, Serialize, Deserialize)]
pub enum Error {
    #[error("Function not found: {0}")]
    FunctionNotFound(FunctionId),

    #[error("Function not initialized: {0}")]
    FunctionNotInitialized(FunctionId),

    #[error("SerializeError")]
    SerializeError(String),

    #[error("DeserializeError")]
    DeserializeError(String),
}
