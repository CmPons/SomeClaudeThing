use proc_macro::TokenStream;
use std::str::FromStr;

/// A much simpler implementation of Serialize derive macro without dependencies
#[proc_macro_derive(Serialize, attributes(fastjson))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    // Parse the input token stream as a string
    let input_str = input.to_string();
    
    // Extract struct/enum name
    let name = extract_name(&input_str);

    // Generate implementation
    if input_str.contains("struct") {
        // Generate struct implementation
        let fields = extract_struct_fields(&input_str);
        generate_struct_serialize(name, fields)
    } else if input_str.contains("enum") {
        // Extract enum variants
        let variants = extract_enum_variants(&input_str);
        generate_enum_serialize(name, variants)
    } else {
        // Error for unsupported types
        TokenStream::from_str("compile_error!(\"Unsupported type for Serialize derive\")").unwrap()
    }
}

/// A much simpler implementation of Deserialize derive macro without dependencies
#[proc_macro_derive(Deserialize, attributes(fastjson))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    // Parse the input token stream as a string
    let input_str = input.to_string();
    
    // Extract struct/enum name
    let name = extract_name(&input_str);

    // Generate implementation
    if input_str.contains("struct") {
        // Generate struct implementation
        let fields = extract_struct_fields(&input_str);
        generate_struct_deserialize(name, fields)
    } else if input_str.contains("enum") {
        // Extract enum variants
        let variants = extract_enum_variants(&input_str);
        generate_enum_deserialize(name, variants)
    } else {
        // Error for unsupported types
        TokenStream::from_str("compile_error!(\"Unsupported type for Deserialize derive\")").unwrap()
    }
}

// Helper functions

fn extract_name(input: &str) -> &str {
    // Skip to struct/enum keyword
    let mut parts = input.split(|c| c == ' ' || c == '\n');
    while let Some(part) = parts.next() {
        if part == "struct" || part == "enum" {
            // The next part should be the name
            if let Some(name) = parts.next() {
                // Remove any whitespace or generic parameters
                return name.trim().split('<').next().unwrap_or("").trim();
            }
        }
    }
    ""
}

// Represents a simple field with name and type
#[derive(Debug, Clone)]
struct Field {
    name: String,
    rename: Option<String>,
    skip: bool,
    skip_if_none: bool,
    is_option: bool,
}

// Represents a enum variant
#[derive(Debug, Clone)]
enum VariantKind {
    Unit,
    Tuple(Vec<String>),  // field types
    Struct(Vec<Field>),  // named fields
}

#[derive(Debug, Clone)]
struct Variant {
    name: String,
    rename: Option<String>,
    kind: VariantKind,
}

fn extract_struct_fields(input: &str) -> Vec<Field> {
    let mut fields = Vec::new();
    
    // Look for the struct body between { and }
    if let Some(body_start) = input.find('{') {
        if let Some(body_end) = input[body_start..].find('}') {
            let body = &input[body_start + 1..body_start + body_end];
            
            // Split by commas to get individual fields
            for field_str in body.split(',') {
                let field_str = field_str.trim();
                if field_str.is_empty() {
                    continue;
                }
                
                // Check for attributes
                let mut skip = false;
                let mut skip_if_none = false;
                let mut rename = None;
                
                if field_str.contains("#[fastjson") {
                    if field_str.contains("skip)") || field_str.contains("skip,") || field_str.contains("skip ]") {
                        skip = true;
                    }
                    if field_str.contains("skip_if_none)") || field_str.contains("skip_if_none,") || field_str.contains("skip_if_none ]") {
                        skip_if_none = true;
                    }
                    if field_str.contains("rename =") {
                        // More robust extraction of rename value
                        let rename_pattern = "rename = \"";
                        if let Some(rename_start) = field_str.find(rename_pattern) {
                            let start_pos = rename_start + rename_pattern.len();
                            let remaining = &field_str[start_pos..];
                            if let Some(rename_end) = remaining.find('\"') {
                                rename = Some(remaining[..rename_end].to_string());
                            }
                        }
                    }
                }

                // Find field name and type
                let mut parts = field_str.trim().splitn(2, ':');
                let name_part = parts.next().unwrap_or("").trim();
                
                // Get actual field name by taking the last part (after any attributes)
                let name = name_part.split_whitespace().last().unwrap_or("").to_string();
                
                if let Some(type_part) = parts.next() {
                    // Check if field is Option<T>
                    let type_str = type_part.trim();
                    let is_option = type_str.starts_with("Option<");
                    
                    
                    fields.push(Field {
                        name,
                        rename,
                        skip,
                        skip_if_none, 
                        is_option,
                    });
                }
            }
        }
    }
    
    fields
}

fn extract_enum_variants(input: &str) -> Vec<Variant> {
    let mut variants = Vec::new();
    
    // Look for the enum body between { and }
    if let Some(body_start) = input.find('{') {
        if let Some(body_end) = input[body_start..].find('}') {
            let body = &input[body_start + 1..body_start + body_end];
            
            // Process the body in chunks to handle attributes correctly
            let mut current_chunk = String::new();
            let mut brace_count = 0;
            let mut paren_count = 0;
            let mut in_attribute = false;
            
            for c in body.chars() {
                // Track if we're inside an attribute: #[...]
                if c == '#' {
                    in_attribute = true;
                }
                if in_attribute && c == ']' {
                    in_attribute = false;
                }
                
                match c {
                    '{' => {
                        brace_count += 1;
                        current_chunk.push(c);
                    },
                    '}' => {
                        brace_count -= 1;
                        current_chunk.push(c);
                    },
                    '(' => {
                        paren_count += 1;
                        current_chunk.push(c);
                    },
                    ')' => {
                        paren_count -= 1;
                        current_chunk.push(c);
                    },
                    ',' => {
                        if brace_count == 0 && paren_count == 0 && !in_attribute {
                            // Process this variant
                            if !current_chunk.trim().is_empty() {
                                let variant = extract_single_variant(&current_chunk);
                                if let Some(v) = variant {
                                    variants.push(v);
                                }
                            }
                            current_chunk.clear();
                        } else {
                            current_chunk.push(c);
                        }
                    },
                    _ => current_chunk.push(c),
                }
            }
            
            // Process the last variant
            if !current_chunk.trim().is_empty() {
                let variant = extract_single_variant(&current_chunk);
                if let Some(v) = variant {
                    variants.push(v);
                }
            }
        }
    }
    
    variants
}

fn extract_single_variant(variant_str: &str) -> Option<Variant> {
    let variant_str = variant_str.trim();
    if variant_str.is_empty() {
        return None;
    }
    
    // Extract attributes from the variant
    let mut rename = None;
    
    // Check for attribute lines
    let lines: Vec<&str> = variant_str.lines().collect();
    let mut variant_def = String::new();
    let mut in_attribute = false;
    
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("#[") {
            in_attribute = true;
        }
        
        if in_attribute {
            if trimmed.contains("fastjson") && trimmed.contains("rename") {
                // Extract rename value - more robust parsing
                let rename_pattern = "rename = \"";
                if let Some(rename_start) = trimmed.find(rename_pattern) {
                    let start_pos = rename_start + rename_pattern.len();
                    let remaining = &trimmed[start_pos..];
                    if let Some(rename_end) = remaining.find('\"') {
                        rename = Some(remaining[..rename_end].to_string());
                    }
                }
            }
            
            if trimmed.ends_with("]") {
                in_attribute = false;
            }
        } else if !trimmed.starts_with("#[") {
            // Add non-attribute lines to the variant definition
            variant_def.push_str(trimmed);
            variant_def.push(' ');
        }
    }
    
    let variant_def = variant_def.trim();
    
    // Extract the variant name and kind
    if variant_def.is_empty() {
        return None;
    }
    
    // Get variant name
    let name_end = variant_def.find('(').unwrap_or_else(|| variant_def.find('{').unwrap_or(variant_def.len()));
    let name = variant_def[..name_end].trim().to_string();
    
    // Determine variant kind
    let kind = if variant_def.contains('(') && !variant_def.contains('{') {
        // It's a tuple variant
        let tuple_start = variant_def.find('(').unwrap_or(0);
        let tuple_end = variant_def.rfind(')').unwrap_or(variant_def.len());
        
        if tuple_start < tuple_end && tuple_start > 0 {
            let tuple_str = &variant_def[tuple_start + 1..tuple_end];
            let types: Vec<String> = tuple_str.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            VariantKind::Tuple(types)
        } else {
            VariantKind::Unit
        }
    } else if variant_def.contains('{') {
        // It's a struct variant
        let fields_start = variant_def.find('{').unwrap_or(0);
        let fields_end = variant_def.rfind('}').unwrap_or(variant_def.len());
        
        if fields_start < fields_end && fields_start > 0 {
            let fields_str = &variant_def[fields_start + 1..fields_end];
            let fields = extract_struct_fields(&format!("struct Dummy {{ {} }}", fields_str));
            VariantKind::Struct(fields)
        } else {
            VariantKind::Unit
        }
    } else {
        // It's a unit variant
        VariantKind::Unit
    };
    
    Some(Variant {
        name,
        rename,
        kind,
    })
}

fn generate_struct_serialize(name: &str, fields: Vec<Field>) -> TokenStream {
    let mut body = String::new();
    
    // Start implementation
    body.push_str(&format!("impl ::fastjson::Serialize for {} {{\n", name));
    body.push_str("    fn serialize(&self) -> ::fastjson::Result<::fastjson::Value> {\n");
    body.push_str("        use std::collections::HashMap;\n");
    body.push_str("        use ::fastjson::Value;\n");
    body.push_str("        \n");
    body.push_str("        let mut map = HashMap::new();\n");
    
    // Add serialization for each field
    for field in fields {
        if field.skip {
            continue;
        }
        
        let field_name = &field.name;
        let ser_name = field.rename.unwrap_or_else(|| field_name.clone());
        
        if field.skip_if_none && field.is_option {
            body.push_str(&format!(
                "        if let Some(val) = &self.{} {{\n", 
                field_name
            ));
            body.push_str(&format!(
                "            map.insert(\"{}\".to_owned(), ::fastjson::Serialize::serialize(val)?);\n", 
                ser_name
            ));
            body.push_str("        }\n");
        } else {
            body.push_str(&format!(
                "        map.insert(\"{}\".to_owned(), ::fastjson::Serialize::serialize(&self.{})?);\n", 
                ser_name, field_name
            ));
        }
    }
    
    // Finalize implementation
    body.push_str("        Ok(Value::Object(map))\n");
    body.push_str("    }\n");
    body.push_str("}");
    
    TokenStream::from_str(&body).unwrap()
}

fn generate_enum_serialize(name: &str, variants: Vec<Variant>) -> TokenStream {
    let mut body = String::new();
    
    // Start implementation
    body.push_str(&format!("impl ::fastjson::Serialize for {} {{\n", name));
    body.push_str("    fn serialize(&self) -> ::fastjson::Result<::fastjson::Value> {\n");
    body.push_str("        use std::collections::HashMap;\n");
    body.push_str("        use ::fastjson::Value;\n");
    body.push_str("        \n");
    
    // For enums, we need to match on references
    body.push_str("        let result = match *self {\n");
    
    // Generate serialization for each variant
    for variant in &variants {
        let variant_name = &variant.name;
        let json_name = variant.rename.clone().unwrap_or_else(|| variant_name.clone());
        
        match &variant.kind {
            VariantKind::Unit => {
                // Unit variant is serialized as a string with the variant name
                body.push_str(&format!("            {}::{} => Ok(Value::String(\"{}\".to_owned())),\n", 
                    name, variant_name, json_name));
            },
            VariantKind::Tuple(types) => {
                // Tuple variant is serialized as an object with type and data fields
                if types.len() == 1 {
                    // Single field tuple variant
                    body.push_str(&format!("            {}::{}(ref value) => {{\n", name, variant_name));
                    body.push_str("                let mut map = HashMap::new();\n");
                    body.push_str(&format!("                map.insert(\"type\".to_owned(), Value::String(\"{}\".to_owned()));\n", json_name));
                    body.push_str("                map.insert(\"data\".to_owned(), Value::Array(vec![::fastjson::Serialize::serialize(value)?]));\n");
                    body.push_str("                Ok(Value::Object(map))\n");
                    body.push_str("            },\n");
                } else {
                    // Multi-field tuple variant
                    let ref_field_names: Vec<String> = (0..types.len())
                        .map(|i| format!("ref value{}", i))
                        .collect();
                    
                    let ref_pattern = ref_field_names.join(", ");
                    body.push_str(&format!("            {}::{}({}) => {{\n", name, variant_name, ref_pattern));
                    body.push_str("                let mut map = HashMap::new();\n");
                    body.push_str(&format!("                map.insert(\"type\".to_owned(), Value::String(\"{}\".to_owned()));\n", json_name));
                    body.push_str("                let mut data = Vec::new();\n");
                    
                    for field_name in &ref_field_names {
                        // Remove "ref " from the field name
                        let clean_name = field_name.replace("ref ", "");
                        body.push_str(&format!("                data.push(::fastjson::Serialize::serialize({})?); // No & needed as we have ref\n", clean_name));
                    }
                    
                    body.push_str("                map.insert(\"data\".to_owned(), Value::Array(data));\n");
                    body.push_str("                Ok(Value::Object(map))\n");
                    body.push_str("            },\n");
                }
            },
            VariantKind::Struct(fields) => {
                // Generate field patterns for destructuring with ref
                let field_patterns: Vec<String> = fields.iter()
                    .map(|field| format!("ref {}", field.name))
                    .collect();
                
                let ref_pattern = field_patterns.join(", ");
                body.push_str(&format!("            {}::{}{{ {} }} => {{\n", name, variant_name, ref_pattern));
                body.push_str("                let mut map = HashMap::new();\n");
                body.push_str(&format!("                map.insert(\"type\".to_owned(), Value::String(\"{}\".to_owned()));\n", json_name));
                
                // Add each field
                for field in fields {
                    if field.skip {
                        continue;
                    }
                    
                    let field_name = &field.name;
                    let ser_name = field.rename.clone().unwrap_or_else(|| field_name.clone());
                    
                    if field.skip_if_none && field.is_option {
                        body.push_str(&format!("                if let Some(val) = {} {{\n", field_name));
                        body.push_str(&format!("                    map.insert(\"{}\".to_owned(), ::fastjson::Serialize::serialize(val)?); // No & needed due to ref pattern\n", ser_name));
                        body.push_str("                }\n");
                    } else {
                        body.push_str(&format!("                map.insert(\"{}\".to_owned(), ::fastjson::Serialize::serialize({})?);\n", 
                            ser_name, field_name));
                    }
                }
                
                body.push_str("                Ok(Value::Object(map))\n");
                body.push_str("            },\n");
            }
        }
    }
    
    // Close match and implementation
    body.push_str("        };\n");
    body.push_str("        result\n");
    body.push_str("    }\n");
    body.push_str("}");
    
    TokenStream::from_str(&body).unwrap()
}

fn generate_struct_deserialize(name: &str, fields: Vec<Field>) -> TokenStream {
    let mut body = String::new();
    
    // Start implementation
    body.push_str(&format!("impl ::fastjson::Deserialize for {} {{\n", name));
    body.push_str("    fn deserialize(value: ::fastjson::Value) -> ::fastjson::Result<Self> {\n");
    body.push_str("        use std::collections::HashMap;\n");
    body.push_str("        use ::fastjson::{Value, Error};\n");
    body.push_str("        \n");
    body.push_str("        match value {\n");
    body.push_str("            Value::Object(map) => {\n");
    
    // Add deserialization for each field
    for field in &fields {
        let field_name = &field.name;
        let ser_name = field.rename.clone().unwrap_or_else(|| field_name.clone());
        
        if field.skip {
            body.push_str(&format!("                let {} = Default::default();\n", field_name));
            continue;
        }
        
        if field.is_option {
            body.push_str(&format!("                let {} = match map.get(\"{}\") {{\n", field_name, ser_name));
            body.push_str("                    Some(v) => {\n");
            body.push_str("                        if v.is_null() {\n");
            body.push_str("                            None\n");
            body.push_str("                        } else {\n");
            body.push_str("                            Some(::fastjson::Deserialize::deserialize(v.clone())?)\n");
            body.push_str("                        }\n");
            body.push_str("                    },\n");
            body.push_str("                    None => None,\n");
            body.push_str("                };\n");
        } else if field.skip_if_none {
            body.push_str(&format!("                let {} = match map.get(\"{}\") {{\n", field_name, ser_name));
            body.push_str("                    Some(v) => ::fastjson::Deserialize::deserialize(v.clone())?,\n");
            body.push_str("                    None => Default::default(),\n");
            body.push_str("                };\n");
        } else {
            body.push_str(&format!("                let {} = match map.get(\"{}\") {{\n", field_name, ser_name));
            body.push_str("                    Some(v) => ::fastjson::Deserialize::deserialize(v.clone())?,\n");
            body.push_str(&format!("                    None => return Err(Error::MissingField(\"{}\".to_string())),\n", ser_name));
            body.push_str("                };\n");
        }
    }
    
    // Create the struct with deserialized fields
    body.push_str("                \n");
    body.push_str("                Ok(Self {\n");
    for field in &fields {
        body.push_str(&format!("                    {},\n", field.name));
    }
    body.push_str("                })\n");
    
    // Finalize implementation
    body.push_str("            },\n");
    body.push_str("            _ => Err(Error::TypeError(format!(\"expected object, found {:?}\", value))),\n");
    body.push_str("        }\n");
    body.push_str("    }\n");
    body.push_str("}");
    
    TokenStream::from_str(&body).unwrap()
}fn generate_enum_deserialize(name: &str, variants: Vec<Variant>) -> TokenStream {
    let mut body = String::new();
    
    // Start implementation
    body.push_str(&format!("impl ::fastjson::Deserialize for {} {{\n", name));
    body.push_str("    fn deserialize(value: ::fastjson::Value) -> ::fastjson::Result<Self> {\n");
    body.push_str("        use ::fastjson::{Value, Error};\n");
    body.push_str("        use std::collections::HashMap;\n");
    body.push_str("        \n");
    
    // First handle strings for unit variants
    body.push_str("        match value {\n");
    body.push_str("            Value::String(s) => {\n");
    body.push_str("                match s.as_str() {\n");
    
    // Handle unit variants
    for variant in &variants {
        if let VariantKind::Unit = variant.kind {
            let variant_name = &variant.name;
            let json_name = variant.rename.clone().unwrap_or_else(|| variant_name.clone());
            body.push_str(&format!("                    \"{}\" => Ok({}::{}),\n", 
                json_name, name, variant_name));
        }
    }
    
    // Handle unknown string variants
    body.push_str("                    _ => Err(Error::TypeError(format!(\"unknown enum variant: {}\", s))),\n");
    body.push_str("                }\n");
    body.push_str("            },\n");
    
    // Handle objects for tuple and struct variants
    body.push_str("            Value::Object(map) => {\n");
    body.push_str("                if let Some(Value::String(t)) = map.get(\"type\") {\n");
    body.push_str("                    match t.as_str() {\n");
    
    // Handle tuple and struct variants
    for variant in &variants {
        let variant_name = &variant.name;
        let json_name = variant.rename.clone().unwrap_or_else(|| variant_name.clone());
        
        match &variant.kind {
            VariantKind::Unit => {
                // Already handled above for string values
            },
            VariantKind::Tuple(types) => {
                body.push_str(&format!("                        \"{}\" => {{\n", json_name));
                body.push_str("                            if let Some(Value::Array(arr)) = map.get(\"data\") {\n");
                
                // Check array length
                body.push_str(&format!("                                if arr.len() != {} {{\n", types.len()));
                body.push_str(&format!("                                    return Err(Error::TypeError(format!(\"expected array with {} element(s), found array with {{}} elements\", arr.len())));\n", types.len()));
                body.push_str("                                }\n");
                
                // Deserialize each field
                if types.len() == 1 {
                    // Single field tuple variant
                    body.push_str("                                let value = ::fastjson::Deserialize::deserialize(arr[0].clone())?;\n");
                    body.push_str(&format!("                                return Ok({}::{}(value));\n", name, variant_name));
                } else {
                    // Multi-field tuple variant
                    for (i, _) in types.iter().enumerate() {
                        body.push_str(&format!("                                let value{} = ::fastjson::Deserialize::deserialize(arr[{}].clone())?;\n", i, i));
                    }
                    
                    let values = (0..types.len()).map(|i| format!("value{}", i)).collect::<Vec<_>>().join(", ");
                    body.push_str(&format!("                                return Ok({}::{}({}));\n", name, variant_name, values));
                }
                
                body.push_str("                            }\n");
                body.push_str("                            Err(Error::TypeError(\"expected array for enum variant data\".to_string()))\n");
                body.push_str("                        },\n");
            },
            VariantKind::Struct(fields) => {
                body.push_str(&format!("                        \"{}\" => {{\n", json_name));
                
                // Deserialize each field
                for field in fields {
                    let field_name = &field.name;
                    let ser_name = field.rename.clone().unwrap_or_else(|| field_name.clone());
                    
                    if field.skip {
                        body.push_str(&format!("                            let {} = Default::default();\n", field_name));
                        continue;
                    }
                    
                    if field.is_option {
                        body.push_str(&format!("                            let {} = match map.get(\"{}\") {{\n", field_name, ser_name));
                        body.push_str("                                Some(v) => {\n");
                        body.push_str("                                    if v.is_null() {\n");
                        body.push_str("                                        None\n");
                        body.push_str("                                    } else {\n");
                        body.push_str("                                        Some(::fastjson::Deserialize::deserialize(v.clone())?)\n");
                        body.push_str("                                    }\n");
                        body.push_str("                                },\n");
                        body.push_str("                                None => None,\n");
                        body.push_str("                            };\n");
                    } else if field.skip_if_none {
                        body.push_str(&format!("                            let {} = match map.get(\"{}\") {{\n", field_name, ser_name));
                        body.push_str("                                Some(v) => ::fastjson::Deserialize::deserialize(v.clone())?,\n");
                        body.push_str("                                None => Default::default(),\n");
                        body.push_str("                            };\n");
                    } else {
                        body.push_str(&format!("                            let {} = match map.get(\"{}\") {{\n", field_name, ser_name));
                        body.push_str("                                Some(v) => ::fastjson::Deserialize::deserialize(v.clone())?,\n");
                        body.push_str(&format!("                                None => return Err(Error::MissingField(\"{}\".to_string())),\n", ser_name));
                        body.push_str("                            };\n");
                    }
                }
                
                // Create the struct variant
                let field_names = fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(", ");
                body.push_str(&format!("                            return Ok({}::{}{{ {} }});\n", name, variant_name, field_names));
                body.push_str("                        },\n");
            }
        }
    }
    
    // Handle unknown variant types
    body.push_str("                        _ => Err(Error::TypeError(format!(\"unknown enum variant type: {}\", t))),\n");
    body.push_str("                    }\n");
    body.push_str("                } else {\n");
    body.push_str("                    Err(Error::MissingField(\"type\".to_string()))\n");
    body.push_str("                }\n");
    body.push_str("            },\n");
    
    // Handle unexpected value types
    body.push_str("            _ => Err(Error::TypeError(format!(\"expected string or object for enum, found {:?}\", value))),\n");
    body.push_str("        }\n");
    body.push_str("    }\n");
    body.push_str("}");
    
    TokenStream::from_str(&body).unwrap()
}