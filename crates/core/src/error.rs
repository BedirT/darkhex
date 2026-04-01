use std::fmt;

/// Core error type for darkhex game engine operations.
#[derive(Debug)]
pub enum CoreError {
    /// Invalid board dimensions or action index.
    InvalidArgument(String),
    /// I/O error during save/load.
    Io(String),
    /// Serialization/deserialization error.
    Serialization(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::InvalidArgument(msg) => write!(f, "invalid argument: {msg}"),
            CoreError::Io(msg) => write!(f, "I/O error: {msg}"),
            CoreError::Serialization(msg) => write!(f, "serialization error: {msg}"),
        }
    }
}

impl std::error::Error for CoreError {}
