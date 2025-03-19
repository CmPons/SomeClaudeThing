/// Code generation module for the fastjson derive macros
/// 
/// This module is responsible for generating the code for the Serialize and Deserialize
/// trait implementations.

use proc_macro::TokenStream;
use std::str::FromStr;
use crate::parser::{Field, Variant, VariantKind};

/// Simple code builder that helps with indentation and code generation
struct CodeBuilder {
    /// The accumulated code
    code: String,
    /// The current indentation level
    indent_level: usize,
    /// The number of spaces per indentation level
    indent_size: usize,
}

impl CodeBuilder {
    /// Creates a new CodeBuilder with default indentation of 4 spaces
    fn new() -> Self {
        Self {
            code: String::new(),
            indent_level: 0,
            indent_size: 4,
        }
    }
    
    /// Helper method to generate option field handling for deserializing Option fields
    fn generate_option_handling(&mut self, indent: &str) -> &mut Self {
        self.line(&format!("{}Some(v) => {{", indent))
            .line(&format!("{}    if v.is_null() {{", indent))
            .line(&format!("{}        None", indent))
            .line(&format!("{}    }} else {{", indent))
            .line(&format!("{}        Some(::fastjson::Deserialize::deserialize(v.clone())?)", indent))
            .line(&format!("{}    }}", indent))
            .line(&format!("{}}},", indent))
            .line(&format!("{}None => None", indent))
    }

    /// Adds a line of code with the current indentation
    fn line(&mut self, line: &str) -> &mut Self {
        let indent = " ".repeat(self.indent_level * self.indent_size);
        self.code.push_str(&indent);
        self.code.push_str(line);
        self.code.push('\n');
        self
    }

    /// Adds multiple lines of code with each line indented
    fn lines(&mut self, lines: &[&str]) -> &mut Self {
        for line in lines {
            self.line(line);
        }
        self
    }

    /// Starts a new block with increased indentation
    fn block<F>(&mut self, block_start: &str, block_end: &str, f: F) -> &mut Self 
    where 
        F: FnOnce(&mut Self),
    {
        self.line(block_start);
        self.indent_level += 1;
        f(self);
        self.indent_level -= 1;
        self.line(block_end);
        self
    }

    /// Returns the generated code as a string
    fn build(&self) -> String {
        // Ensure all brackets are balanced. Insert a note if there's a mismatch
        let mut bracket_count = 0;
        let mut paren_count = 0;
        let mut brace_count = 0;
        
        for c in self.code.chars() {
            match c {
                '{' => bracket_count += 1,
                '}' => bracket_count -= 1,
                '(' => paren_count += 1,
                ')' => paren_count -= 1,
                '[' => brace_count += 1,
                ']' => brace_count -= 1,
                _ => {}
            }
        }
        
        // Add debug information if there's an imbalance
        let mut result = self.code.clone();
        if bracket_count != 0 || paren_count != 0 || brace_count != 0 {
            result.push_str(&format!("\n// WARNING: Unbalanced delimiters: {} braces, {} parentheses, {} brackets\n", 
                bracket_count, paren_count, brace_count));
        }
        
        result
    }
}

/// Generates the Serialize implementation for a struct
pub fn generate_struct_serialize(name: &str, fields: Vec<Field>) -> TokenStream {
    let mut builder = CodeBuilder::new();

    // Start implementation
    builder.line(&format!("impl ::fastjson::Serialize for {} {{", name))
        .block("    fn serialize(&self) -> ::fastjson::Result<::fastjson::Value> {", "    }", |b| {
            b.lines(&[
                "use std::collections::HashMap;",
                "use ::fastjson::Value;",
                "",
                "let mut map = HashMap::new();"
            ]);

            // Add serialization for each field
            for field in &fields {
                if field.skip {
                    continue;
                }
                
                let field_name = &field.name;
                let ser_name = field.rename.as_ref().unwrap_or(field_name);
                
                if field.skip_if_none && field.is_option {
                    b.block(&format!("if let Some(val) = &self.{} {{", field_name), "}", |b| {
                        b.line(&format!("map.insert(\"{}\".to_owned(), ::fastjson::Serialize::serialize(val)?);", ser_name));
                    });
                } else {
                    b.line(&format!("map.insert(\"{}\".to_owned(), ::fastjson::Serialize::serialize(&self.{})?);"
                        , ser_name, field_name));
                }
            }
            
            // Return the result
            b.line("Ok(Value::Object(map))");
        });

    TokenStream::from_str(&builder.build()).unwrap()
}

/// Generates the Deserialize implementation for a struct
pub fn generate_struct_deserialize(name: &str, fields: Vec<Field>) -> TokenStream {
    let mut builder = CodeBuilder::new();

    // Start implementation
    builder.line(&format!("impl ::fastjson::Deserialize for {} {{", name))
        .block("    fn deserialize(value: ::fastjson::Value) -> ::fastjson::Result<Self> {", "    }", |b| {
            b.lines(&[
                "use std::collections::HashMap;",
                "use ::fastjson::{Value, Error};",
                ""
            ]);

            // Match on the value
            b.block("match value {", "}", |b| {
                b.block("Value::Object(map) => {", "}", |b| {
                    // Deserialize each field
                    for field in &fields {
                        let field_name = &field.name;
                        let ser_name = field.rename.as_ref().unwrap_or(field_name);
                        
                        if field.skip {
                            b.line(&format!("let {} = Default::default();", field_name));
                            continue;
                        }
                        
                        b.line(&format!("let {} = match map.get(\"{}\") {{", field_name, ser_name));
                        b.indent_level += 1;
                        
                        if field.is_option {
                            b.generate_option_handling("");
                        } else if field.skip_if_none {
                            b.line("Some(v) => ::fastjson::Deserialize::deserialize(v.clone())?,");
                            b.line("None => Default::default()");
                        } else {
                            b.line("Some(v) => ::fastjson::Deserialize::deserialize(v.clone())?,");
                            b.line(&format!("None => return Err(Error::MissingField(\"{}\".to_string())),", ser_name));
                        }
                        
                        b.indent_level -= 1;
                        b.line("};");
                    }
                    
                    // Create the struct with deserialized fields
                    b.line("")
                     .block("Ok(Self {", "})", |b| {
                        for field in &fields {
                            b.line(&format!("{},", field.name));
                        }
                    });
                })
                .line("_ => Err(Error::TypeError(format!(\"expected object, found {:?}\", value))),");
            });
        });

    TokenStream::from_str(&builder.build()).unwrap()
}

/// Generates the Serialize implementation for an enum
pub fn generate_enum_serialize(name: &str, variants: Vec<Variant>) -> TokenStream {
    let mut builder = CodeBuilder::new();

    // Start implementation
    builder.line(&format!("impl ::fastjson::Serialize for {} {{", name))
        .block("    fn serialize(&self) -> ::fastjson::Result<::fastjson::Value> {", "    }", |b| {
            b.lines(&[
                "use std::collections::HashMap;",
                "use ::fastjson::Value;",
                ""
            ]);

            // Match the enum directly instead of using *self which is causing issues
            b.block("match self {", "}", |b| {
                // Generate match arms for each variant
                for variant in &variants {
                    let variant_name = &variant.name;
                    let json_name = variant.rename.as_ref().unwrap_or(variant_name);
                    
                    match &variant.kind {
                        VariantKind::Unit => {
                            // Unit variant is serialized as a string with the variant name
                            b.line(&format!("{}::{} => Ok(Value::String(\"{}\".to_owned())),", 
                                name, variant_name, json_name));
                        },
                        VariantKind::Tuple(types) => {
                            // Tuple variant is serialized as an object with type and data fields
                            if types.len() == 1 {
                                // Single field tuple variant
                                b.block(&format!("{}::{}(value) => {{", name, variant_name), "}", |b| {
                                    b.line("let mut map = HashMap::new();")
                                     .line(&format!("map.insert(\"type\".to_owned(), Value::String(\"{}\".to_owned()));", json_name))
                                     .line("map.insert(\"data\".to_owned(), Value::Array(vec![::fastjson::Serialize::serialize(value)?]));")
                                     .line("Ok(Value::Object(map))");
                                });
                            } else {
                                // Multi-field tuple variant
                                let fields: Vec<String> = (0..types.len())
                                    .map(|i| format!("value{}", i))
                                    .collect();
                                
                                b.block(&format!("{}::{}({}) => {{", name, variant_name, fields.join(", ")), "}", |b| {
                                    b.line("let mut map = HashMap::new();")
                                     .line(&format!("map.insert(\"type\".to_owned(), Value::String(\"{}\".to_owned()));", json_name))
                                     .line("let mut data = Vec::new();");
                                    
                                    // Serialize each field
                                    for i in 0..types.len() {
                                        b.line(&format!("data.push(::fastjson::Serialize::serialize(&value{})?);", i));
                                    }
                                    
                                    b.line("map.insert(\"data\".to_owned(), Value::Array(data));")
                                     .line("Ok(Value::Object(map))");
                                });
                            }
                        },
                        VariantKind::Struct(fields) => {
                            // Struct variant with fields
                            let field_names = fields.iter()
                                .map(|f| f.name.clone())
                                .collect::<Vec<_>>()
                                .join(", ");
                            
                            b.block(&format!("{}::{}{{ {} }} => {{", name, variant_name, field_names), "}", |b| {
                                b.line("let mut map = HashMap::new();")
                                 .line(&format!("map.insert(\"type\".to_owned(), Value::String(\"{}\".to_owned()));", json_name));
                                
                                // Serialize each field
                                for field in fields {
                                    if field.skip {
                                        continue;
                                    }
                                    
                                    let field_name = &field.name;
                                    let ser_name = field.rename.as_ref().unwrap_or(field_name);
                                    
                                    if field.skip_if_none && field.is_option {
                                        b.block(&format!("if let Some(ref val) = {} {{", field_name), "}", |b| {
                                            b.line(&format!("map.insert(\"{}\".to_owned(), ::fastjson::Serialize::serialize(val)?);", ser_name));
                                        });
                                    } else {
                                        b.line(&format!("map.insert(\"{}\".to_owned(), ::fastjson::Serialize::serialize(&{})?);"
                                            , ser_name, field_name));
                                    }
                                }
                                
                                b.line("Ok(Value::Object(map))");
                            });
                        }
                    }
                }
            });
        });

    TokenStream::from_str(&builder.build()).unwrap()
}

/// Generates the Deserialize implementation for an enum
pub fn generate_enum_deserialize(name: &str, variants: Vec<Variant>) -> TokenStream {
    let mut builder = CodeBuilder::new();

    // Start implementation
    builder.line(&format!("impl ::fastjson::Deserialize for {} {{", name))
        .block("    fn deserialize(value: ::fastjson::Value) -> ::fastjson::Result<Self> {", "    }", |b| {
            b.lines(&[
                "use ::fastjson::{Value, Error};",
                "use std::collections::HashMap;",
                ""
            ]);

            // Match on the value
            b.block("match value {", "}", |b| {
                // Handle strings (unit variants)
                b.block("Value::String(s) => {", "}", |b| {
                    b.block("match s.as_str() {", "}", |b| {
                        // Generate match arms for unit variants
                        for variant in &variants {
                            if let VariantKind::Unit = variant.kind {
                                let variant_name = &variant.name;
                                let json_name = variant.rename.as_ref().unwrap_or(variant_name);
                                b.line(&format!("\"{}\" => Ok({}::{}),", json_name, name, variant_name));
                            }
                        }
                        
                        // Handle unknown variants
                        b.line("_ => Err(Error::TypeError(format!(\"unknown enum variant: {}\", s))),");
                    });
                });
                
                // Handle objects (tuple and struct variants)
                b.block("Value::Object(map) => {", "}", |b| {
                    b.line("if let Some(Value::String(t)) = map.get(\"type\") {");
                    b.indent_level += 1;
                    b.line("match t.as_str() {");
                    b.indent_level += 1;
                    
                    // Generate match arms for tuple and struct variants
                    for variant in &variants {
                        if let VariantKind::Unit = variant.kind {
                            continue;  // Already handled above
                        }
                        
                        let variant_name = &variant.name;
                        let json_name = variant.rename.as_ref().unwrap_or(variant_name);
                        
                        match &variant.kind {
                            VariantKind::Unit => {
                                // Already handled above
                            },
                            VariantKind::Tuple(types) => {
                                b.line(&format!("\"{}\" => {{", json_name));
                                b.indent_level += 1;
                                b.line("if let Some(Value::Array(arr)) = map.get(\"data\") {");
                                b.indent_level += 1;
                                
                                // Check array length
                                b.line(&format!("if arr.len() == {} {{", types.len()));
                                b.indent_level += 1;
                                
                                // Deserialize based on number of elements
                                if types.len() == 1 {
                                    // Single field tuple variant
                                    b.line("let value = ::fastjson::Deserialize::deserialize(arr[0].clone())?;")
                                     .line(&format!("return Ok({}::{}(value));", name, variant_name));
                                } else {
                                    // Multi-field tuple variant
                                    for i in 0..types.len() {
                                        b.line(&format!("let value{} = ::fastjson::Deserialize::deserialize(arr[{}].clone())?;", i, i));
                                    }
                                    
                                    let values = (0..types.len())
                                        .map(|i| format!("value{}", i))
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    
                                    b.line(&format!("return Ok({}::{}({}));", name, variant_name, values));
                                }
                                
                                b.indent_level -= 1;
                                b.line("} else {");
                                b.indent_level += 1;
                                b.line(&format!("return Err(Error::TypeError(format!(\"expected array with {} element(s), found array with {{}} elements\", arr.len())));", types.len()));
                                b.indent_level -= 1;
                                b.line("}");
                                
                                b.indent_level -= 1;
                                b.line("} else {");
                                b.indent_level += 1;
                                b.line("return Err(Error::TypeError(\"expected array for enum variant data\".to_string()));");
                                b.indent_level -= 1;
                                b.line("}");
                                
                                b.indent_level -= 1;
                                b.line("}");
                            },
                            VariantKind::Struct(fields) => {
                                b.line(&format!("\"{}\" => {{", json_name));
                                b.indent_level += 1;
                                
                                // Deserialize each field
                                for field in fields {
                                    let field_name = &field.name;
                                    let ser_name = field.rename.as_ref().unwrap_or(field_name);
                                    
                                    if field.skip {
                                        b.line(&format!("let {} = Default::default();", field_name));
                                        continue;
                                    }
                                    
                                    b.line(&format!("let {} = match map.get(\"{}\") {{", field_name, ser_name));
                                    b.indent_level += 1;
                                    
                                    if field.is_option {
                                        b.generate_option_handling("");
                                    } else if field.skip_if_none {
                                        b.line("Some(v) => ::fastjson::Deserialize::deserialize(v.clone())?,");
                                        b.line("None => Default::default()");
                                    } else {
                                        b.line("Some(v) => ::fastjson::Deserialize::deserialize(v.clone())?,");
                                        b.line(&format!("None => return Err(Error::MissingField(\"{}\".to_string())),", ser_name));
                                    }
                                    
                                    b.indent_level -= 1;
                                    b.line("};");
                                }
                                
                                // Create the struct variant
                                let field_names = fields.iter()
                                    .map(|f| f.name.clone())
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                
                                b.line("");
                                b.line(&format!("return Ok({}::{}{{ {} }});", name, variant_name, field_names));
                                
                                b.indent_level -= 1;
                                b.line("}");
                            }
                        }
                    }
                    
                    // Handle unknown variant types
                    b.line("_ => Err(Error::TypeError(format!(\"unknown enum variant type: {}\", t))),");
                    
                    b.indent_level -= 1;
                    b.line("}");
                    b.indent_level -= 1;
                    b.line("} else {");
                    b.line("    Err(Error::MissingField(\"type\".to_string()))");
                    b.line("}");
                });
                
                // Handle other types
                b.line("_ => Err(Error::TypeError(format!(\"expected string or object for enum, found {:?}\", value))),");
            });
        });

    TokenStream::from_str(&builder.build()).unwrap()
}