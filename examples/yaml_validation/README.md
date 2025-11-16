# YAML Validation Example

This example demonstrates JSON Schema-style validation for YAML documents using the `yaml_lib` validation module.

## Features Demonstrated

1. **Simple Type Validation** - Validate basic types (string, number, boolean, etc.)
2. **Object with Required Fields** - Define required properties with constraints
3. **Array Validation** - Validate array elements against schemas
4. **Nested Object Validation** - Validate deeply nested structures
5. **Range and Length Constraints** - Apply minimum/maximum values and lengths
6. **Pattern and Enum Validation** - Match patterns and restrict to allowed values
7. **Custom Validators** - Create custom validation logic
8. **Real-World Configuration** - Validate complex application configuration

## Running the Example

```bash
cargo run --release
```

## Key Concepts

### Schema Definition

Define schemas using `PropertySchema` with various constraints:

```rust
let schema = PropertySchema::new(SchemaType::String)
    .required()
    .with_min_length(3)
    .with_max_length(50)
    .with_pattern("@");
```

### Object Schemas

Define object schemas with nested properties:

```rust
let mut properties = BTreeMap::new();
properties.insert("name".to_string(), PropertySchema::new(SchemaType::String).required());
properties.insert("age".to_string(), PropertySchema::new(SchemaType::Integer));

let schema = Schema::object(properties);
```

### Array Schemas

Validate arrays with element schemas:

```rust
let schema = Schema::array(PropertySchema::new(SchemaType::Integer));
```

### Validation

Use `SchemaValidator` to validate nodes:

```rust
let validator = SchemaValidator::new(schema);

match validator.validate(&node) {
    Ok(_) => println!("Valid!"),
    Err(errors) => {
        for error in errors {
            println!("{}: {}", error.path, error.message);
        }
    }
}
```

## Validation Features

- **Type Checking** - Ensure nodes match expected types
- **Range Validation** - Minimum and maximum values for numbers
- **Length Validation** - Minimum and maximum lengths for strings and arrays
- **Pattern Matching** - Simple substring pattern matching for strings
- **Enum Validation** - Restrict values to allowed set
- **Required Fields** - Mark object properties as required
- **Nested Validation** - Validate nested objects and arrays
- **Error Paths** - Get detailed paths to validation errors (e.g., `$.user.address.street`)
- **Custom Validators** - Implement custom validation logic

## Schema Types

- `SchemaType::String` - String values
- `SchemaType::Number` - Any numeric value
- `SchemaType::Integer` - Integer values only
- `SchemaType::Float` - Float values only
- `SchemaType::Boolean` - Boolean values
- `SchemaType::Null` - Null values
- `SchemaType::Array` - Arrays/sequences
- `SchemaType::Object` - Objects/mappings
- `SchemaType::Any` - Any type allowed

## Example Output

```
Example 1: Simple Type Validation
-----------------------------------
✓ Valid string accepted
✓ Correctly rejected number: Type mismatch: expected String, got "Number"

Example 2: Object with Required Fields
---------------------------------------
✓ Valid user object accepted
✓ Correctly rejected: Required field 'name' is missing

...

Example 8: Real-World Configuration Validation
-----------------------------------------------
✓ Valid configuration accepted
✓ Found 1 validation error(s):
  - $.database: Required field 'driver' is missing
```

## Use Cases

- **Configuration Validation** - Validate application configuration files
- **API Input Validation** - Validate API request payloads
- **Data Quality** - Ensure data meets expected structure and constraints
- **Schema Enforcement** - Enforce schemas on YAML documents
- **Documentation** - Use schemas to document expected structure

## See Also

- [Schema Types](../../library/src/validation/schema.rs)
- [Built-in Validators](../../library/src/validation/validators.rs)
- [Validation Engine](../../library/src/validation/engine.rs)
