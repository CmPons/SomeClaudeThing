use std::fmt;

/// Error that can occur during serialization or deserialization
#[derive(Debug, PartialEq)]
pub enum Error {
    /// I/O error with a message
    Io(String),

    /// Unexpected end of input
    Eof,

    /// Invalid syntax error at specific position
    Syntax {
        position: usize,
        message: String,
    },

    /// Expected a certain token but found something else
    ExpectedFound {
        expected: &'static str,
        found: String,
        position: usize,
    },

    /// Missing required field
    MissingField(String),

    /// Unknown field
    UnknownField(String),

    /// Type error (type mismatch)
    TypeError(String),

    /// Custom error with message
    Custom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(msg) => write!(f, "I/O error: {}", msg),
            Error::Eof => write!(f, "Unexpected end of input"),
            Error::Syntax { position, message } => {
                write!(f, "Invalid syntax at position {}: {}", position, message)
            }
            Error::ExpectedFound { expected, found, position } => {
                write!(f, "Expected {} but found {} at position {}", expected, found, position)
            }
            Error::MissingField(field) => write!(f, "Missing field: {}", field),
            Error::UnknownField(field) => write!(f, "Unknown field: {}", field),
            Error::TypeError(msg) => write!(f, "Type error: {}", msg),
            Error::Custom(msg) => write!(f, "Custom error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

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