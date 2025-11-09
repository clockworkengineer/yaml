# YAML Error Handling Example

This example demonstrates robust error handling patterns when working with YAML data. It shows how to gracefully handle parse errors, validate structures, and implement defensive programming practices.

## What This Example Shows

- **Parse error handling** - Catching and reporting syntax errors
- **Structure validation** - Verifying data matches expected schema
- **File operation errors** - Handling I/O failures
- **Safe document access** - Preventing out-of-bounds errors
- **Defensive traversal** - Safely accessing nested values
- **Error recovery** - Strategies for continuing after errors

## Error Types

The library can encounter several types of errors:

### Parse Errors
- Syntax errors (invalid YAML)
- Indentation problems
- Unclosed quotes or brackets
- Invalid anchor references
- Invalid tag usage

### Structure Errors
- Missing required fields
- Wrong data types
- Invalid values
- Schema violations

### I/O Errors
- File not found
- Permission denied
- Read/write failures
- Encoding issues

## Usage

```bash
# Run the example
cargo run --example yaml_error_handling

# Or from the example directory
cd examples/yaml_error_handling
cargo run
```

## Examples Demonstrated

### 1. Parse Error Handling

Shows how to handle various YAML syntax errors:

```rust
let mut source = BufferSource::new(yaml.as_bytes());
match parse(&mut source) {
    Ok(node) => {
        println!("✓ Parsed successfully");
        // Process the node
    }
    Err(e) => {
        println!("✗ Parse error: {}", e);
        // Handle the error:
        // - Log it
        // - Use default configuration
        // - Show user-friendly message
        // - Attempt auto-correction
    }
}
```

### 2. Structure Validation

Validates that parsed YAML matches expected structure:

```rust
fn validate_user_structure(node: &Node) -> Result<(), String> {
    let obj = match node {
        Node::Object(map) => map,
        _ => return Err("Root must be an object".to_string()),
    };
    
    // Check required fields
    let required_fields = vec!["name", "age", "email"];
    for field in required_fields {
        if !obj.contains_key(field) {
            return Err(format!("Missing required field: {}", field));
        }
    }
    
    // Validate types and values
    if let Some(Node::String(email)) = obj.get("email") {
        if !email.contains('@') {
            return Err("Invalid email format".to_string());
        }
    }
    
    Ok(())
}
```

### 3. File Error Handling

Handles file system errors gracefully:

```rust
match FileSource::new("config.yaml") {
    Ok(mut source) => {
        match parse(&mut source) {
            Ok(node) => process_config(&node),
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }
    Err(e) => {
        eprintln!("File error: {}", e);
        // Use default configuration
        use_default_config();
    }
}
```

### 4. Safe Document Access

Prevents panics when accessing multi-document streams:

```rust
// Always check document count first
let count = get_number_of_documents(&node);

for i in 0..count {
    match get_document(&node, i) {
        Ok(doc) => process_document(&doc),
        Err(e) => eprintln!("Document {} error: {}", i, e),
    }
}

// Or check specific index
match get_document(&node, 5) {
    Ok(doc) => println!("Document found"),
    Err(e) => println!("Document not found: {}", e),
}
```

### 5. Defensive Node Traversal

Safely navigates nested structures:

```rust
fn get_nested_string<'a>(node: &'a Node, path: &[&str]) -> Option<&'a str> {
    let mut current = node;
    
    for key in path {
        match current {
            Node::Object(map) => {
                current = map.get(*key)?;
            }
            _ => return None,
        }
    }
    
    match current {
        Node::String(s) => Some(s),
        _ => None,
    }
}

// Usage
let name = get_nested_string(&node, &["user", "profile", "name"]);
println!("Name: {}", name.unwrap_or("Unknown"));
```

### 6. Error Recovery

Implements fallback strategies:

```rust
let config = match parse(&mut source) {
    Ok(node) => {
        // Validate and sanitize
        sanitize_config(node)
    }
    Err(e) => {
        eprintln!("Parse failed: {}, using defaults", e);
        get_default_config()
    }
};
```

## Best Practices

### 1. Always Handle Errors

```rust
// ✗ Bad: Unwrapping can panic
let node = parse(&mut source).unwrap();

// ✓ Good: Handle errors explicitly
let node = match parse(&mut source) {
    Ok(n) => n,
    Err(e) => {
        eprintln!("Error: {}", e);
        return;
    }
};
```

### 2. Validate Early

```rust
fn process_config(yaml: &str) -> Result<(), String> {
    // Parse
    let mut source = BufferSource::new(yaml.as_bytes());
    let node = parse(&mut source)
        .map_err(|e| format!("Parse error: {}", e))?;
    
    // Validate immediately
    validate_structure(&node)?;
    
    // Then process
    apply_config(&node)?;
    
    Ok(())
}
```

### 3. Provide Context

```rust
// ✗ Bad: Generic error
Err("Invalid data".to_string())

// ✓ Good: Specific context
Err(format!(
    "Invalid email '{}' in user record at line {}",
    email, line_num
))
```

### 4. Use Type-Safe Wrappers

```rust
struct Config {
    database_host: String,
    database_port: u16,
    timeout: u32,
}

impl Config {
    fn from_node(node: &Node) -> Result<Self, String> {
        let obj = node.as_object()
            .ok_or("Config must be an object")?;
        
        let host = obj.get("database_host")
            .and_then(|n| n.as_string())
            .ok_or("Missing database_host")?
            .to_string();
        
        let port = obj.get("database_port")
            .and_then(|n| n.as_integer())
            .ok_or("Missing database_port")? as u16;
        
        Ok(Config {
            database_host: host,
            database_port: port,
            timeout: 30, // default
        })
    }
}
```

### 5. Implement Fallbacks

```rust
fn load_config(path: &str) -> Config {
    FileSource::new(path)
        .and_then(|mut src| parse(&mut src))
        .and_then(|node| Config::from_node(&node))
        .unwrap_or_else(|e| {
            eprintln!("Config error: {}, using defaults", e);
            Config::default()
        })
}
```

## Error Recovery Patterns

### Pattern 1: Default Values

```rust
let timeout = config.get("timeout")
    .and_then(|n| n.as_integer())
    .unwrap_or(30);
```

### Pattern 2: Skip Invalid Items

```rust
let valid_users: Vec<User> = users_node
    .as_array()
    .map(|arr| {
        arr.iter()
            .filter_map(|node| User::from_node(node).ok())
            .collect()
    })
    .unwrap_or_default();
```

### Pattern 3: Sanitization

```rust
fn sanitize_string(node: &Node) -> String {
    match node {
        Node::String(s) => s.trim().to_string(),
        Node::Number(n) => n.to_string(),
        Node::Boolean(b) => b.to_string(),
        _ => String::new(),
    }
}
```

### Pattern 4: Progressive Validation

```rust
fn load_config(node: &Node) -> Result<Config, Vec<String>> {
    let mut errors = Vec::new();
    let mut config = Config::default();
    
    // Collect all errors instead of failing fast
    if let Err(e) = load_database_config(&node, &mut config) {
        errors.push(e);
    }
    
    if let Err(e) = load_server_config(&node, &mut config) {
        errors.push(e);
    }
    
    if errors.is_empty() {
        Ok(config)
    } else {
        Err(errors)
    }
}
```

## Testing Error Conditions

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_invalid_yaml() {
        let yaml = "invalid: yaml: syntax:";
        let mut source = BufferSource::new(yaml.as_bytes());
        assert!(parse(&mut source).is_err());
    }
    
    #[test]
    fn test_missing_required_field() {
        let yaml = "name: John";
        let mut source = BufferSource::new(yaml.as_bytes());
        let node = parse(&mut source).unwrap();
        assert!(validate_user_structure(&node).is_err());
    }
    
    #[test]
    fn test_invalid_type() {
        let yaml = "age: not_a_number";
        let mut source = BufferSource::new(yaml.as_bytes());
        let node = parse(&mut source).unwrap();
        assert!(validate_user_structure(&node).is_err());
    }
}
```

## Common Pitfalls

### 1. Unwrapping Without Checking
```rust
// ✗ Can panic
let value = node.as_object().unwrap().get("key").unwrap();

// ✓ Safe
let value = node.as_object()
    .and_then(|obj| obj.get("key"));
```

### 2. Ignoring Parse Errors
```rust
// ✗ Silently fails
let _ = parse(&mut source);

// ✓ Handle or propagate
parse(&mut source)?;
```

### 3. Not Validating After Parsing
```rust
// ✗ Assumes structure is correct
let node = parse(&mut source)?;
let age = node.as_object().unwrap()["age"];

// ✓ Validate first
let node = parse(&mut source)?;
validate_structure(&node)?;
let age = extract_age(&node)?;
```

## See Also

- **yaml_parse_and_stringify** - Basic YAML operations
- **yaml_node_manipulation** - Safe node creation
- **yaml_multi_document** - Multi-document error handling
