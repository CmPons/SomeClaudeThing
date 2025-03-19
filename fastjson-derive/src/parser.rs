/// Parser module for the fastjson derive macros
/// 
/// This module is responsible for parsing the input token stream and extracting
/// the necessary information for generating the serialization and deserialization
/// implementations.

/// Represents the type of input being parsed
#[derive(Debug)]
pub enum InputType {
    /// A struct with a name and fields
    Struct {
        name: String,
        fields: Vec<Field>,
    },
    /// An enum with a name and variants
    Enum {
        name: String,
        variants: Vec<Variant>,
    },
    /// An unknown or unsupported type
    Unknown,
}

impl std::fmt::Display for InputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputType::Struct { name, .. } => write!(f, "Struct {}", name),
            InputType::Enum { name, .. } => write!(f, "Enum {}", name),
            InputType::Unknown => write!(f, "Unknown Type"),
        }
    }
}

/// Represents a field in a struct or a struct variant
#[derive(Debug, Clone)]
pub struct Field {
    /// The name of the field in the Rust code
    pub name: String,
    /// The name to use in the JSON output (if different)
    pub rename: Option<String>,
    /// Whether to skip this field during serialization
    pub skip: bool,
    /// Whether to skip this field if it is None (for Option<T> fields)
    pub skip_if_none: bool,
    /// Whether this field is an Option<T>
    pub is_option: bool,
}

/// Represents the kind of enum variant
#[derive(Debug, Clone)]
pub enum VariantKind {
    /// A simple unit variant (e.g., `Variant`)
    Unit,
    /// A tuple variant (e.g., `Variant(T1, T2, ...)`)
    Tuple(Vec<String>),
    /// A struct variant (e.g., `Variant { field1: T1, ... }`)
    Struct(Vec<Field>),
}

/// Represents an enum variant
#[derive(Debug, Clone)]
pub struct Variant {
    /// The name of the variant in the Rust code
    pub name: String,
    /// The name to use in the JSON output (if different)
    pub rename: Option<String>,
    /// The kind of variant (unit, tuple, or struct)
    pub kind: VariantKind,
}

/// Parses the input string into the appropriate type
pub fn parse_input(input: &str) -> InputType {
    // Extract the name of the type
    let name = extract_name(input);
    
    // Determine if this is a struct or an enum
    if input.contains("struct") {
        let fields = extract_struct_fields(input);
        InputType::Struct {
            name: name.to_string(),
            fields,
        }
    } else if input.contains("enum") {
        let variants = extract_enum_variants(input);
        InputType::Enum {
            name: name.to_string(),
            variants,
        }
    } else {
        InputType::Unknown
    }
}

/// Extracts the name of the type from the input
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

/// Extracts the fields from a struct definition
pub fn extract_struct_fields(input: &str) -> Vec<Field> {
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
                
                // Extract field information
                if let Some(field) = extract_field(field_str) {
                    fields.push(field);
                }
            }
        }
    }
    
    fields
}

/// Extracts a single field from a field string
fn extract_field(field_str: &str) -> Option<Field> {
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
            // Extract rename value
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
        
        Some(Field {
            name,
            rename,
            skip,
            skip_if_none, 
            is_option,
        })
    } else {
        None
    }
}

/// Extracts the variants from an enum definition
pub fn extract_enum_variants(input: &str) -> Vec<Variant> {
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
                                if let Some(variant) = extract_single_variant(&current_chunk) {
                                    variants.push(variant);
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
                if let Some(variant) = extract_single_variant(&current_chunk) {
                    variants.push(variant);
                }
            }
        }
    }
    
    variants
}

/// Extracts a single variant from a variant string
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