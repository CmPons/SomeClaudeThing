use thiserror::Error;
use std::fmt;

/// Error that can occur during serialization or deserialization
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("Unexpected end of input")]
    Eof,

    #[error("Invalid syntax at position {position}: {message}")]
    Syntax {
        position: usize,
        message: String,
    },

    #[error("Expected {expected} but found {found} at position {position}")]
    ExpectedFound {
        expected: &'static str,
        found: String,
        position: usize,
    },

    #[error("Missing field: {0}")]
    MissingField(String),

    #[error("Unknown field: {0}")]
    UnknownField(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Custom error: {0}")]
    Custom(String),
}

/// Result type for serialization and deserialization operations
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }

    pub fn syntax<T: fmt::Display>(position: usize, msg: T) -> Self {
        Error::Syntax {
            position,
            message: msg.to_string(),
        }
    }

    pub fn expected_found(expected: &'static str, found: impl fmt::Display, position: usize) -> Self {
        Error::ExpectedFound {
            expected,
            found: found.to_string(),
            position,
        }
    }
}