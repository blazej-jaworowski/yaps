use std::string::FromUtf8Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("FunctionNotFound error: {0}")]
    FunctionNotFound(String),

    #[error("FromUtf8Error error: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("SerializeError")]
    SerializeError(String),

    #[error("DeserializeError")]
    DeserializeError(String),
}
