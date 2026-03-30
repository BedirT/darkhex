use std::fmt;

#[derive(Debug)]
pub enum CoreError {
    InvalidDimension(String),
    InvalidAction(String),
    GameTerminal,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::InvalidDimension(msg) => write!(f, "{msg}"),
            CoreError::InvalidAction(msg) => write!(f, "{msg}"),
            CoreError::GameTerminal => write!(f, "game is already terminal"),
        }
    }
}

impl std::error::Error for CoreError {}
