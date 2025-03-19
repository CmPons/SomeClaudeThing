use crate::error::{Error, Result};
use crate::value::Value;
use std::collections::HashMap;
use std::str::FromStr;

/// A trait for types that can be deserialized from JSON
pub trait Deserialize: Sized {
    /// Deserialize this value from JSON
    fn deserialize(value: Value) -> Result<Self>;
}

impl Deserialize for bool {
    fn deserialize(value: Value) -> Result<Self> {
        match value {
            Value::Bool(b) => Ok(b),
            _ => Err(Error::TypeError(format!("expected boolean, found {:?}", value))),
        }
    }
}

impl Deserialize for i8 {
    fn deserialize(value: Value) -> Result<Self> {
        match value {
            Value::Number(n) => {
                if n.fract() != 0.0 {
                    return Err(Error::TypeError(format!("expected integer, found float {}", n)));
                }
                if n < i8::MIN as f64 || n > i8::MAX as f64 {
                    return Err(Error::TypeError(format!("value {} out of range for i8", n)));
                }
                Ok(n as i8)
            }
            _ => Err(Error::TypeError(format!("expected number, found {:?}", value))),
        }
    }
}

impl Deserialize for i16 {
    fn deserialize(value: Value) -> Result<Self> {
        match value {
            Value::Number(n) => {
                if n.fract() != 0.0 {
                    return Err(Error::TypeError(format!("expected integer, found float {}", n)));
                }
                if n < i16::MIN as f64 || n > i16::MAX as f64 {
                    return Err(Error::TypeError(format!("value {} out of range for i16", n)));
                }
                Ok(n as i16)
            }
            _ => Err(Error::TypeError(format!("expected number, found {:?}", value))),
        }
    }
}

impl Deserialize for i32 {
    fn deserialize(value: Value) -> Result<Self> {
        match value {
            Value::Number(n) => {
                if n.fract() != 0.0 {
                    return Err(Error::TypeError(format!("expected integer, found float {}", n)));
                }
                if n < i32::MIN as f64 || n > i32::MAX as f64 {
                    return Err(Error::TypeError(format!("value {} out of range for i32", n)));
                }
                Ok(n as i32)
            }
            _ => Err(Error::TypeError(format!("expected number, found {:?}", value))),
        }
    }
}

impl Deserialize for i64 {
    fn deserialize(value: Value) -> Result<Self> {
        match value {
            Value::Number(n) => {
                if n.fract() != 0.0 {
                    return Err(Error::TypeError(format!("expected integer, found float {}", n)));
                }
                // JavaScript can't precisely represent all i64 values, so we need to check if this 
                // value is accurately representable as an i64
                if n < -9007199254740991.0 || n > 9007199254740991.0 {
                    return Err(Error::TypeError(format!(
                        "value {} may not be precisely representable as i64", n
                    )));
                }
                Ok(n as i64)
            }
            _ => Err(Error::TypeError(format!("expected number, found {:?}", value))),
        }
    }
}

impl Deserialize for u8 {
    fn deserialize(value: Value) -> Result<Self> {
        match value {
            Value::Number(n) => {
                if n.fract() != 0.0 {
                    return Err(Error::TypeError(format!("expected integer, found float {}", n)));
                }
                if n < 0.0 || n > u8::MAX as f64 {
                    return Err(Error::TypeError(format!("value {} out of range for u8", n)));
                }
                Ok(n as u8)
            }
            _ => Err(Error::TypeError(format!("expected number, found {:?}", value))),
        }
    }
}

impl Deserialize for u16 {
    fn deserialize(value: Value) -> Result<Self> {
        match value {
            Value::Number(n) => {
                if n.fract() != 0.0 {
                    return Err(Error::TypeError(format!("expected integer, found float {}", n)));
                }
                if n < 0.0 || n > u16::MAX as f64 {
                    return Err(Error::TypeError(format!("value {} out of range for u16", n)));
                }
                Ok(n as u16)
            }
            _ => Err(Error::TypeError(format!("expected number, found {:?}", value))),
        }
    }
}

impl Deserialize for u32 {
    fn deserialize(value: Value) -> Result<Self> {
        match value {
            Value::Number(n) => {
                if n.fract() != 0.0 {
                    return Err(Error::TypeError(format!("expected integer, found float {}", n)));
                }
                if n < 0.0 || n > u32::MAX as f64 {
                    return Err(Error::TypeError(format!("value {} out of range for u32", n)));
                }
                Ok(n as u32)
            }
            _ => Err(Error::TypeError(format!("expected number, found {:?}", value))),
        }
    }
}

impl Deserialize for u64 {
    fn deserialize(value: Value) -> Result<Self> {
        match value {
            Value::Number(n) => {
                if n.fract() != 0.0 {
                    return Err(Error::TypeError(format!("expected integer, found float {}", n)));
                }
                if n < 0.0 {
                    return Err(Error::TypeError(format!("value {} out of range for u64", n)));
                }
                // JavaScript can't precisely represent all u64 values, so we need to check if this 
                // value is accurately representable as a u64
                if n > 9007199254740991.0 {
                    return Err(Error::TypeError(format!(
                        "value {} may not be precisely representable as u64", n
                    )));
                }
                Ok(n as u64)
            }
            _ => Err(Error::TypeError(format!("expected number, found {:?}", value))),
        }
    }
}

impl Deserialize for f32 {
    fn deserialize(value: Value) -> Result<Self> {
        match value {
            Value::Number(n) => Ok(n as f32),
            _ => Err(Error::TypeError(format!("expected number, found {:?}", value))),
        }
    }
}

impl Deserialize for f64 {
    fn deserialize(value: Value) -> Result<Self> {
        match value {
            Value::Number(n) => Ok(n),
            _ => Err(Error::TypeError(format!("expected number, found {:?}", value))),
        }
    }
}

impl Deserialize for String {
    fn deserialize(value: Value) -> Result<Self> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(Error::TypeError(format!("expected string, found {:?}", value))),
        }
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    fn deserialize(value: Value) -> Result<Self> {
        if let Value::Null = value {
            Ok(None)
        } else {
            Ok(Some(T::deserialize(value)?))
        }
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize(value: Value) -> Result<Self> {
        match value {
            Value::Array(arr) => {
                let mut result = Vec::with_capacity(arr.len());
                for item in arr {
                    result.push(T::deserialize(item)?);
                }
                Ok(result)
            }
            _ => Err(Error::TypeError(format!("expected array, found {:?}", value))),
        }
    }
}

impl<K, V> Deserialize for HashMap<K, V>
where
    K: FromStr + std::hash::Hash + Eq,
    V: Deserialize,
{
    fn deserialize(value: Value) -> Result<Self> {
        match value {
            Value::Object(map) => {
                let mut result = HashMap::with_capacity(map.len());
                for (key, value) in map {
                    match K::from_str(&key) {
                        Ok(k) => result.insert(k, V::deserialize(value)?),
                        Err(_) => return Err(Error::TypeError(format!("invalid key: {}", key))),
                    };
                }
                Ok(result)
            }
            _ => Err(Error::TypeError(format!("expected object, found {:?}", value))),
        }
    }
}

impl Deserialize for Value {
    fn deserialize(value: Value) -> Result<Self> {
        Ok(value)
    }
}

// Parse a JSON string into a Value
pub fn parse(json: &str) -> Result<Value> {
    let mut parser = Parser::new(json);
    let value = parser.parse()?;
    
    // Make sure we've consumed all input
    parser.skip_whitespace();
    if parser.peek().is_some() {
        // Character position for error
        let (pos, c) = parser.peek().unwrap();
        return Err(Error::syntax(pos, format!("trailing character '{}' after JSON value", c)));
    }
    
    Ok(value)
}

// Deserialize a JSON string into any type that implements Deserialize
pub fn from_str<T: Deserialize>(json: &str) -> Result<T> {
    let value = parse(json)?;
    T::deserialize(value)
}

// JSON parser
struct Parser<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            pos: 0,
        }
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        self.chars.peek().cloned()
    }

    fn next(&mut self) -> Option<(usize, char)> {
        let next = self.chars.next();
        if let Some((pos, _)) = next {
            self.pos = pos;
        }
        next
    }

    fn skip_whitespace(&mut self) {
        while let Some((_, ch)) = self.peek() {
            if !ch.is_whitespace() {
                break;
            }
            self.next();
        }
    }

    fn parse(&mut self) -> Result<Value> {
        self.skip_whitespace();
        self.parse_value()
    }

    fn parse_value(&mut self) -> Result<Value> {
        self.skip_whitespace();
        
        // Get the current character and position
        let (pos, c) = match self.peek() {
            Some(p) => p,
            None => return Err(Error::Eof),
        };
        
        // Dispatch to the appropriate parser based on the first character
        match c {
            'n' => self.parse_null(),
            't' => self.parse_true(),
            'f' => self.parse_false(),
            '"' => self.parse_string(),
            '[' => {
                // Special handling for array
                let value = self.parse_array();
                if value.is_err() {
                    // Show detailed error message
                    if let Err(err) = &value {
                        Err(Error::syntax(pos, format!("Failed to parse array: {}", err)))
                    } else {
                        value
                    }
                } else {
                    value
                }
            },
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(Error::syntax(pos, format!("unexpected character: {}", c))),
        }
    }
    
    // Split bool into two functions for clarity
    fn parse_true(&mut self) -> Result<Value> {
        let pos = self.pos;
        if self.pos + 4 <= self.input.len() && &self.input[self.pos..self.pos+4] == "true" {
            for _ in 0..4 {
                self.next();
            }
            Ok(Value::Bool(true))
        } else {
            Err(Error::syntax(pos, "expected 'true'"))
        }
    }
    
    fn parse_false(&mut self) -> Result<Value> {
        let pos = self.pos;
        if self.pos + 5 <= self.input.len() && &self.input[self.pos..self.pos+5] == "false" {
            for _ in 0..5 {
                self.next();
            }
            Ok(Value::Bool(false))
        } else {
            Err(Error::syntax(pos, "expected 'false'"))
        }
    }

    fn parse_null(&mut self) -> Result<Value> {
        let current_pos = self.pos;
        
        if self.input[current_pos..].starts_with("null") {
            for _ in 0..4 {
                self.next();
            }
            Ok(Value::Null)
        } else {
            Err(Error::syntax(current_pos, "expected 'null'"))
        }
    }

    #[allow(dead_code)]
    fn parse_bool(&mut self) -> Result<Value> {
        let current_pos = self.pos;
        
        // Check for true
        if self.input[current_pos..].starts_with("true") {
            for _ in 0..4 {
                self.next();
            }
            return Ok(Value::Bool(true));
        }
        
        // Check for false
        if self.input[current_pos..].starts_with("false") {
            for _ in 0..5 {
                self.next();
            }
            return Ok(Value::Bool(false));
        }
        
        // Neither true nor false
        Err(Error::syntax(current_pos, "expected 'true' or 'false'"))
    }

    fn parse_string(&mut self) -> Result<Value> {
        self.next(); // Skip opening quote
        
        let mut result = String::new();
        let mut escaped = false;
        
        loop {
            match self.next() {
                Some((_, '"')) if !escaped => break,
                Some((_, '\\')) if !escaped => escaped = true,
                Some((_, 'n')) if escaped => {
                    result.push('\n');
                    escaped = false;
                }
                Some((_, 'r')) if escaped => {
                    result.push('\r');
                    escaped = false;
                }
                Some((_, 't')) if escaped => {
                    result.push('\t');
                    escaped = false;
                }
                Some((_, 'b')) if escaped => {
                    result.push('\u{0008}');
                    escaped = false;
                }
                Some((_, 'f')) if escaped => {
                    result.push('\u{000C}');
                    escaped = false;
                }
                Some((_, '"')) if escaped => {
                    result.push('"');
                    escaped = false;
                }
                Some((_, '\\')) if escaped => {
                    result.push('\\');
                    escaped = false;
                }
                Some((_, 'u')) if escaped => {
                    // Parse unicode escape \uXXXX
                    let mut code_point = 0;
                    for _ in 0..4 {
                        match self.next() {
                            Some((_, c)) if c.is_ascii_hexdigit() => {
                                code_point = code_point * 16 + c.to_digit(16).unwrap();
                            }
                            Some((p, c)) => {
                                return Err(Error::syntax(p, format!("invalid unicode escape: {}", c)));
                            }
                            None => return Err(Error::Eof),
                        }
                    }
                    
                    match std::char::from_u32(code_point) {
                        Some(c) => result.push(c),
                        None => return Err(Error::syntax(self.pos, "invalid unicode code point")),
                    }
                    
                    escaped = false;
                }
                Some((pos, c)) if escaped => {
                    return Err(Error::syntax(pos, format!("invalid escape: \\{}", c)));
                }
                Some((_, c)) => {
                    result.push(c);
                }
                None => return Err(Error::Eof),
            }
        }
        
        Ok(Value::String(result))
    }

    fn parse_number(&mut self) -> Result<Value> {
        let mut number_str = String::new();
        let start_pos = self.pos;
        
        // Check for negative sign
        if let Some((_, '-')) = self.peek() {
            number_str.push('-');
            self.next();
        }
        
        // Parse integer part
        let mut has_digits = false;
        
        // Handle leading zero
        if let Some((_, '0')) = self.peek() {
            number_str.push('0');
            has_digits = true;
            self.next();
        } else {
            // Handle other digits
            while let Some((_, c)) = self.peek() {
                if !c.is_ascii_digit() {
                    break;
                }
                number_str.push(c);
                has_digits = true;
                self.next();
            }
        }
        
        if !has_digits {
            return Err(Error::syntax(start_pos, "expected digit"));
        }
        
        // Parse fractional part
        if let Some((_, '.')) = self.peek() {
            number_str.push('.');
            self.next();
            
            let mut has_fractional_digits = false;
            while let Some((_, c)) = self.peek() {
                if !c.is_ascii_digit() {
                    break;
                }
                number_str.push(c);
                has_fractional_digits = true;
                self.next();
            }
            
            if !has_fractional_digits {
                return Err(Error::syntax(self.pos, "expected digit after decimal point"));
            }
        }
        
        // Parse exponent
        if let Some((_, e)) = self.peek() {
            if e == 'e' || e == 'E' {
                number_str.push(e);
                self.next();
                
                // Check for exponent sign
                if let Some((_, s)) = self.peek() {
                    if s == '+' || s == '-' {
                        number_str.push(s);
                        self.next();
                    }
                }
                
                let mut has_exponent_digits = false;
                while let Some((_, c)) = self.peek() {
                    if !c.is_ascii_digit() {
                        break;
                    }
                    number_str.push(c);
                    has_exponent_digits = true;
                    self.next();
                }
                
                if !has_exponent_digits {
                    return Err(Error::syntax(self.pos, "expected digit in exponent"));
                }
            }
        }
        
        // Parse the number string
        match number_str.parse::<f64>() {
            Ok(n) => Ok(Value::Number(n)),
            Err(_) => Err(Error::syntax(start_pos, format!("invalid number: {}", number_str))),
        }
    }

    fn parse_array(&mut self) -> Result<Value> {
        self.next(); // Skip opening bracket
        self.skip_whitespace();
        
        let mut items = Vec::new();
        
        // Check for empty array
        if let Some((_, ']')) = self.peek() {
            self.next();
            return Ok(Value::Array(items));
        }
        
        // Parse first item
        items.push(self.parse_value()?);
        self.skip_whitespace();
        
        // Parse remaining items
        loop {
            match self.peek() {
                Some((_, ',')) => {
                    self.next();
                    self.skip_whitespace();
                    
                    // JSON doesn't allow trailing commas, so this is an error
                    if let Some((pos, ']')) = self.peek() {
                        return Err(Error::syntax(pos, "trailing comma in array is not allowed in JSON"));
                    }
                    
                    // Parse value after comma
                    items.push(self.parse_value()?);
                    self.skip_whitespace();
                }
                Some((_, ']')) => {
                    self.next();
                    break;
                }
                Some((pos, c)) => {
                    return Err(Error::expected_found("',' or ']'", c, pos));
                }
                None => return Err(Error::Eof),
            }
        }
        
        Ok(Value::Array(items))
    }

    fn parse_object(&mut self) -> Result<Value> {
        self.next(); // Skip opening brace
        self.skip_whitespace();
        
        let mut map = HashMap::new();
        
        // Check for empty object
        if let Some((_, '}')) = self.peek() {
            self.next();
            return Ok(Value::Object(map));
        }
        
        // First key-value pair
        if let Some((_, '"')) = self.peek() {
            // Parse key as string
            let key_value = self.parse_string()?;
            let key = match key_value {
                Value::String(s) => s,
                _ => unreachable!(), // This should never happen since we just parsed a string
            };
            
            // Expect colon
            self.skip_whitespace();
            match self.peek() {
                Some((_, ':')) => {
                    self.next();
                }
                Some((pos, c)) => {
                    return Err(Error::expected_found("':'", c, pos));
                }
                None => return Err(Error::Eof),
            }
            
            // Parse value (skip whitespace before value)
            self.skip_whitespace();
            let value = self.parse_value()?;
            
            // Insert key-value pair
            map.insert(key, value);
            self.skip_whitespace();
        } else if let Some((pos, c)) = self.peek() {
            return Err(Error::expected_found("'\"' or '}'", c, pos));
        } else {
            return Err(Error::Eof);
        }
        
        // Remaining key-value pairs
        loop {
            match self.peek() {
                Some((_, ',')) => {
                    self.next();
                    self.skip_whitespace();
                    
                    // JSON doesn't allow trailing commas, so this is an error
                    if let Some((pos, '}')) = self.peek() {
                        return Err(Error::syntax(pos, "trailing comma in object is not allowed in JSON"));
                    }
                    
                    // println!("Position after comma: {}", self.pos);
                    
                    // Parse key
                    if let Some((_, '"')) = self.peek() {
                        // Parse key as string
                        let key_value = self.parse_string()?;
                        let key = match key_value {
                            Value::String(s) => s,
                            _ => unreachable!(), // This should never happen since we just parsed a string
                        };
                        
                        // Expect colon
                        self.skip_whitespace();
                        match self.peek() {
                            Some((_, ':')) => {
                                self.next();
                            }
                            Some((pos, c)) => {
                                return Err(Error::expected_found("':'", c, pos));
                            }
                            None => return Err(Error::Eof),
                        }
                        
                        // Parse value (skip whitespace before value)
                        self.skip_whitespace();
                        let value = self.parse_value()?;
                        
                        // Insert key-value pair
                        map.insert(key, value);
                        self.skip_whitespace();
                    } else if let Some((pos, c)) = self.peek() {
                        return Err(Error::expected_found("'\"'", c, pos));
                    } else {
                        return Err(Error::Eof);
                    }
                }
                Some((_, '}')) => {
                    self.next();
                    break;
                }
                Some((pos, c)) => {
                    return Err(Error::expected_found("',' or '}'", c, pos));
                }
                None => return Err(Error::Eof),
            }
        }
        
        Ok(Value::Object(map))
    }
}