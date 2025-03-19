fn generate_enum_deserialize(name: &str, variants: Vec<Variant>) -> TokenStream {
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
            let json_name = variant.rename.as_ref().unwrap_or(variant_name);
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
        let json_name = variant.rename.as_ref().unwrap_or(variant_name);
        
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
                    let ser_name = field.rename.as_ref().unwrap_or(field_name);
                    
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