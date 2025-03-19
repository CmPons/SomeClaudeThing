use std::collections::HashMap;
use std::fmt;

/// Represents any valid JSON value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// JSON null
    Null,
    /// JSON boolean
    Bool(bool),
    /// JSON number (stored as f64 for simplicity)
    Number(f64),
    /// JSON string
    String(String),
    /// JSON array
    Array(Vec<Value>),
    /// JSON object
    Object(HashMap<String, Value>),
}

impl Value {
    /// Returns true if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns true if the value is a boolean
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    /// Returns true if the value is a number
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Returns true if the value is a string
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Returns true if the value is an array
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// Returns true if the value is an object
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    /// Try to get this value as a boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get this value as a number
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to get this value as a string reference
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get this value as an array reference
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Try to get this value as a mutable array reference
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Try to get this value as an object reference
    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Try to get this value as a mutable object reference
    pub fn as_object_mut(&mut self) -> Option<&mut HashMap<String, Value>> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Index into an array or object
    pub fn get(&self, index: impl Index) -> Option<&Value> {
        index.index_into(self)
    }
}

/// Types that can be used to index into a `Value`
pub trait Index {
    /// Return a reference to the value at the index if it exists
    fn index_into(self, value: &Value) -> Option<&Value>;
}

impl Index for usize {
    fn index_into(self, value: &Value) -> Option<&Value> {
        match value {
            Value::Array(array) => array.get(self),
            _ => None,
        }
    }
}

impl<'a> Index for &'a str {
    fn index_into(self, value: &Value) -> Option<&Value> {
        match value {
            Value::Object(map) => map.get(self),
            _ => None,
        }
    }
}

impl Index for String {
    fn index_into(self, value: &Value) -> Option<&Value> {
        self.as_str().index_into(value)
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

// Display implementation for debugging
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", escape_string(s)),
            Value::Array(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Object(o) => {
                write!(f, "{{")?;
                for (i, (k, v)) in o.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", escape_string(k), v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

// Helper function to escape special characters in strings
fn escape_string(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{0008}' => escaped.push_str("\\b"),
            '\u{000C}' => escaped.push_str("\\f"),
            _ => escaped.push(c),
        }
    }
    escaped
}