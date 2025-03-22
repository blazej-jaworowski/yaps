use std::string::FromUtf8Error;
use crate::orchestrator::PluginId;
use crate::plugin_connector::FunctionId;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Function not found: {0}")]
    FunctionNotFound(FunctionId),

    #[error("Plugin not found: {0}")]
    PluginNotFound(PluginId),

    #[error("Plugin already registered: {0}")]
    PluginRegistered(PluginId),

    #[error("FromUtf8Error error: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("SerializeError")]
    SerializeError(String),

    #[error("DeserializeError")]
    DeserializeError(String),
}
