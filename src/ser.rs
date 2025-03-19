use crate::error::{Error, Result};
use crate::value::Value;
use std::collections::HashMap;

/// A trait for types that can be serialized to JSON
pub trait Serialize {
    /// Serialize this value into JSON
    fn serialize(&self) -> Result<Value>;
}

impl Serialize for bool {
    fn serialize(&self) -> Result<Value> {
        Ok(Value::Bool(*self))
    }
}

impl Serialize for i8 {
    fn serialize(&self) -> Result<Value> {
        Ok(Value::Number(*self as f64))
    }
}

impl Serialize for i16 {
    fn serialize(&self) -> Result<Value> {
        Ok(Value::Number(*self as f64))
    }
}

impl Serialize for i32 {
    fn serialize(&self) -> Result<Value> {
        Ok(Value::Number(*self as f64))
    }
}

impl Serialize for i64 {
    fn serialize(&self) -> Result<Value> {
        Ok(Value::Number(*self as f64))
    }
}

impl Serialize for u8 {
    fn serialize(&self) -> Result<Value> {
        Ok(Value::Number(*self as f64))
    }
}

impl Serialize for u16 {
    fn serialize(&self) -> Result<Value> {
        Ok(Value::Number(*self as f64))
    }
}

impl Serialize for u32 {
    fn serialize(&self) -> Result<Value> {
        Ok(Value::Number(*self as f64))
    }
}

impl Serialize for u64 {
    fn serialize(&self) -> Result<Value> {
        // JSON doesn't support 64-bit integers precisely, so check for overflow
        if *self > 9007199254740991 { // 2^53 - 1, largest integer precisely representable in f64
            return Err(Error::custom(format!("integer too large for JSON: {}", self)));
        }
        Ok(Value::Number(*self as f64))
    }
}

impl Serialize for f32 {
    fn serialize(&self) -> Result<Value> {
        Ok(Value::Number(*self as f64))
    }
}

impl Serialize for f64 {
    fn serialize(&self) -> Result<Value> {
        if self.is_finite() {
            Ok(Value::Number(*self))
        } else {
            Err(Error::custom(format!("non-finite number cannot be serialized: {}", self)))
        }
    }
}

impl Serialize for str {
    fn serialize(&self) -> Result<Value> {
        Ok(Value::String(self.to_owned()))
    }
}

impl Serialize for String {
    fn serialize(&self) -> Result<Value> {
        Ok(Value::String(self.clone()))
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn serialize(&self) -> Result<Value> {
        match self {
            Some(value) => value.serialize(),
            None => Ok(Value::Null),
        }
    }
}

impl<T: Serialize> Serialize for [T] {
    fn serialize(&self) -> Result<Value> {
        let mut vec = Vec::with_capacity(self.len());
        for item in self {
            vec.push(item.serialize()?);
        }
        Ok(Value::Array(vec))
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self) -> Result<Value> {
        <[T] as Serialize>::serialize(self)
    }
}

impl<K: AsRef<str>, V: Serialize> Serialize for HashMap<K, V> {
    fn serialize(&self) -> Result<Value> {
        let mut map = HashMap::with_capacity(self.len());
        for (key, value) in self {
            map.insert(key.as_ref().to_owned(), value.serialize()?);
        }
        Ok(Value::Object(map))
    }
}

impl<T: Serialize> Serialize for &T {
    fn serialize(&self) -> Result<Value> {
        (*self).serialize()
    }
}

impl Serialize for Value {
    fn serialize(&self) -> Result<Value> {
        Ok(self.clone())
    }
}

// Serializes any value to a JSON string
pub fn to_string<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    let value = value.serialize()?;
    Ok(value.to_string())
}

// Serializes any value to a pretty-printed JSON string with indentation
pub fn to_string_pretty<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    let value = value.serialize()?;
    pretty_print(&value, 0)
}

fn pretty_print(value: &Value, indent: usize) -> Result<String> {
    match value {
        Value::Null => Ok("null".to_owned()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Number(n) => Ok(n.to_string()),
        Value::String(s) => {
            let escaped = s.chars()
                .map(|c| match c {
                    '"' => "\\\"".to_owned(),
                    '\\' => "\\\\".to_owned(),
                    '\n' => "\\n".to_owned(),
                    '\r' => "\\r".to_owned(),
                    '\t' => "\\t".to_owned(),
                    '\u{0008}' => "\\b".to_owned(),
                    '\u{000C}' => "\\f".to_owned(),
                    _ => c.to_string(),
                })
                .collect::<Vec<_>>()
                .join("");
            Ok(format!("\"{}\"", escaped))
        },
        Value::Array(a) => {
            if a.is_empty() {
                return Ok("[]".to_owned());
            }
            
            let next_indent = indent + 2;
            let mut result = String::from("[\n");
            
            for (i, item) in a.iter().enumerate() {
                result.push_str(&" ".repeat(next_indent));
                result.push_str(&pretty_print(item, next_indent)?);
                
                if i < a.len() - 1 {
                    result.push_str(",\n");
                } else {
                    result.push('\n');
                }
            }
            
            result.push_str(&" ".repeat(indent));
            result.push(']');
            Ok(result)
        },
        Value::Object(o) => {
            if o.is_empty() {
                return Ok("{}".to_owned());
            }
            
            let next_indent = indent + 2;
            let mut result = String::from("{\n");
            
            let len = o.len();
            for (i, (key, value)) in o.iter().enumerate() {
                result.push_str(&" ".repeat(next_indent));
                result.push('"');
                result.push_str(key);
                result.push_str("\": ");
                result.push_str(&pretty_print(value, next_indent)?);
                
                if i < len - 1 {
                    result.push_str(",\n");
                } else {
                    result.push('\n');
                }
            }
            
            result.push_str(&" ".repeat(indent));
            result.push('}');
            Ok(result)
        }
    }
}