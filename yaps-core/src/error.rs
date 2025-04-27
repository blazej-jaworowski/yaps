use serde::{Deserialize, Serialize}; // TODO: This should probably be enabled by a feature

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Plugin not initialized: {0}")]
    PluginNotInitialized(String),

    #[error("Plugin wrapper dropped: {0}")]
    PluginWrapperDropped(String),

    #[error("Function not initialized: {0}")]
    FunctionNotInitialized(String),

    #[error("Encode error")]
    Encode(String),

    #[error("Decode error")]
    Decode(String),

    #[error("Channel send error: {0}")]
    ChannelSend(String),

    #[error("Function handler invalidated")]
    HandlerInvalidated,
}
