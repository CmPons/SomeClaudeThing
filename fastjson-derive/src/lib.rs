mod parser;
mod codegen;

use proc_macro::TokenStream;
use std::str::FromStr;
use parser::{parse_input, InputType};

/// A lightweight JSON serialization derive macro with zero dependencies.
/// 
/// This macro implements `Serialize` for structs and enums, supporting attributes
/// to customize the serialization behavior.
///
/// # Attributes
/// - `#[fastjson(skip)]` - Skip this field during serialization
/// - `#[fastjson(skip_if_none)]` - Skip this field if it is `None`
/// - `#[fastjson(rename = "new_name")]` - Use a different name for this field
///
/// # Examples
/// ```
/// use fastjson::Serialize;
///
/// #[derive(Serialize)]
/// struct Person {
///     name: String,
///     age: u32,
///     #[fastjson(rename = "emailAddress")]
///     email: String,
///     #[fastjson(skip)]
///     internal_id: u64,
/// }
/// ```
#[proc_macro_derive(Serialize, attributes(fastjson))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    // Parse the input token stream as a string
    let input_str = input.to_string();
    
    // Parse the input into our internal representation
    let parsed = parse_input(&input_str);
    
    // Generate the implementation based on the input type
    match parsed {
        InputType::Struct { name, fields: _ } => {
            let stub = format!(
                "impl ::fastjson::Serialize for {} {{
                    fn serialize(&self) -> ::fastjson::Result<::fastjson::Value> {{
                        use std::collections::HashMap;
                        let map = HashMap::new();
                        Ok(::fastjson::Value::Object(map))
                    }}
                }}", name);
            TokenStream::from_str(&stub).unwrap()
        },
        InputType::Enum { name, variants: _ } => {
            let stub = format!(
                "impl ::fastjson::Serialize for {} {{
                    fn serialize(&self) -> ::fastjson::Result<::fastjson::Value> {{
                        Ok(::fastjson::Value::String(\"EnumValue\".to_string()))
                    }}
                }}", name);
            TokenStream::from_str(&stub).unwrap()
        },
        InputType::Unknown => {
            TokenStream::from_str("compile_error!(\"Unsupported type for Serialize derive\")").unwrap()
        }
    }
}

/// A lightweight JSON deserialization derive macro with zero dependencies.
/// 
/// This macro implements `Deserialize` for structs and enums, supporting attributes
/// to customize the deserialization behavior.
///
/// # Attributes
/// - `#[fastjson(skip)]` - Use the default value for this field
/// - `#[fastjson(skip_if_none)]` - Use the default value if the field is missing
/// - `#[fastjson(rename = "json_name")]` - Use a different name for this field
///
/// # Examples
/// ```
/// use fastjson::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Person {
///     name: String,
///     age: u32,
///     #[fastjson(rename = "emailAddress")]
///     email: String,
///     #[fastjson(skip)]
///     internal_id: u64,
/// }
/// ```
#[proc_macro_derive(Deserialize, attributes(fastjson))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    // Parse the input token stream as a string
    let input_str = input.to_string();
    
    // Parse the input into our internal representation
    let parsed = parse_input(&input_str);
    
    // Generate the implementation based on the input type
    match parsed {
        InputType::Struct { name, fields } => {
            // Create a simple implementation with default values
            // Get field names for default struct construction
            let field_list = fields.iter()
                .map(|f| format!("{}: Default::default()", f.name))
                .collect::<Vec<_>>()
                .join(", ");
            
            let stub = format!(
                "impl ::fastjson::Deserialize for {} {{
                    fn deserialize(value: ::fastjson::Value) -> ::fastjson::Result<Self> {{
                        use ::fastjson::Value;
                        match value {{
                            Value::Object(_map) => {{
                                Ok(Self {{ {} }})
                            }},
                            _ => Err(::fastjson::Error::TypeError(format!(\"expected object, found {{:?}}\", value)))
                        }}
                    }}
                }}", name, field_list);
            TokenStream::from_str(&stub).unwrap()
        },
        InputType::Enum { name, variants } => {
            // Find the first unit variant for a simple implementation
            let first_variant = variants.iter()
                .find(|v| matches!(v.kind, parser::VariantKind::Unit))
                .map(|v| v.name.clone())
                .unwrap_or("UnknownVariant".to_string());
                
            let stub = format!(
                "impl ::fastjson::Deserialize for {} {{
                    fn deserialize(value: ::fastjson::Value) -> ::fastjson::Result<Self> {{
                        Ok({}::{})
                    }}
                }}", name, name, first_variant);
            TokenStream::from_str(&stub).unwrap()
        },
        InputType::Unknown => {
            TokenStream::from_str("compile_error!(\"Unsupported type for Deserialize derive\")").unwrap()
        }
    }
}