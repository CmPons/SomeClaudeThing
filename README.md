# FastJSON

A lightweight JSON serialization/deserialization library with a focus on zero dependencies and fast compilation times.

## Features

- **Zero dependencies**: Only uses `thiserror` for error handling
- **Fast compilation**: Designed to be lightweight and compile quickly
- **Derive macros**: Support for `#[derive(Serialize, Deserialize)]`
- **Customizable**: Field rename, skip, and conditional serialization
- **Detailed errors**: Clear error messages for parsing and type mismatches
- **Zero-copy parsing**: Efficient deserialization that minimizes allocations
- **Support for standard Rust types**: Works with primitive types, collections, and custom structs/enums

## Usage

Add FastJSON to your `Cargo.toml`:

```toml
[dependencies]
fastjson = "0.1.0"
```

### Basic Example

```rust
use fastjson::{Serialize, Deserialize, from_str, to_string};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Person {
    name: String,
    age: u32,
    is_active: bool,
    #[fastjson(rename = "emailAddress")]
    email: Option<String>,
}

// Serialization
let person = Person {
    name: "John Doe".to_string(),
    age: 30,
    is_active: true,
    email: Some("john@example.com".to_string()),
};

let json = to_string(&person).unwrap();
println!("{}", json);

// Deserialization
let parsed: Person = from_str(&json).unwrap();
assert_eq!(parsed, person);
```

### Field Attributes

FastJSON supports several attributes to customize serialization and deserialization:

- `#[fastjson(rename = "newName")]`: Use a different field name in the JSON representation
- `#[fastjson(skip)]`: Skip this field during serialization and deserialization
- `#[fastjson(skip_if_none)]`: Only include this field in serialized output if it's not `None`

### Enum Support

FastJSON can handle Rust enums with different representation strategies:

```rust
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Status {
    Active,                                // Serializes as a string: "Active"
    Pending(String),                       // Serializes as an object with type and data fields
    Custom { code: u32, message: String }, // Serializes as an object with fields
}
```

## Error Handling

FastJSON provides detailed error messages for common issues:

- Missing required fields
- Type mismatches
- Syntax errors with position information
- Range validation for numeric types

## Performance

FastJSON is designed to be reasonably fast while maintaining a small dependency footprint. Benchmarks comparing it to other JSON libraries can be run with:

```
cargo bench
```

## Testing

Run the test suite with:

```
cargo test
```

## License

MIT
