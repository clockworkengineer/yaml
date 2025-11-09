//! Example demonstrating robust error handling with YAML operations
//!
//! This example shows how to:
//! - Handle parse errors gracefully
//! - Validate YAML structure
//! - Recover from errors
//! - Provide helpful error messages
//! - Implement defensive programming patterns

use yaml_lib::{get_document, parse, stringify, BufferDestination, BufferSource, FileSource, Node};

fn main() {
    println!("=== YAML Error Handling Example ===\n");

    // Example 1: Handling parse errors
    demo_parse_errors();

    // Example 2: Validating structure
    demo_structure_validation();

    // Example 3: Handling file errors
    demo_file_errors();

    // Example 4: Safe document access
    demo_safe_document_access();

    // Example 5: Defensive node traversal
    demo_defensive_traversal();

    // Example 6: Error recovery strategies
    demo_error_recovery();
}

/// Demonstrates handling various parse errors
fn demo_parse_errors() {
    println!("--- Example 1: Handling Parse Errors ---");

    let test_cases = vec![
        (
            "Valid YAML",
            r#"
name: John Doe
age: 30
"#,
        ),
        (
            "Invalid indentation",
            r#"
name: John Doe
  age: 30
    invalid: indent
"#,
        ),
        (
            "Unclosed quotes",
            r#"
name: "John Doe
age: 30
"#,
        ),
        (
            "Invalid anchor",
            r#"
base: &anchor
  value: 1
reference: *nonexistent
"#,
        ),
    ];

    for (description, yaml) in test_cases {
        println!("\nTesting: {}", description);
        let mut source = BufferSource::new(yaml.as_bytes());

        match parse(&mut source) {
            Ok(node) => {
                println!("✓ Parsed successfully");
                if let Node::Mapping(map) = &node {
                    println!("  Found {} top-level keys", map.len());
                }
            }
            Err(e) => {
                println!("✗ Parse error: {}", e);
                // In a real application, you might:
                // - Log the error
                // - Return a default value
                // - Show user-friendly message
                // - Attempt to fix common issues
            }
        }
    }
    println!();
}

/// Demonstrates structure validation
fn demo_structure_validation() {
    println!("--- Example 2: Structure Validation ---");

    let yaml = r#"
name: John Doe
age: 30
email: john@example.com
address:
  street: 123 Main St
  city: Springfield
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("✓ Parsed successfully");

            // Validate required fields
            match validate_user_structure(&node) {
                Ok(()) => println!("✓ Structure validation passed"),
                Err(e) => println!("✗ Structure validation failed: {}", e),
            }
        }
        Err(e) => println!("✗ Parse error: {}", e),
    }

    // Test with invalid structure
    let invalid_yaml = r#"
name: John Doe
# Missing required field: age
email: john@example.com
"#;

    let mut source = BufferSource::new(invalid_yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("\n✓ Parsed successfully");
            match validate_user_structure(&node) {
                Ok(()) => println!("✓ Structure validation passed"),
                Err(e) => println!("✗ Structure validation failed: {}", e),
            }
        }
        Err(e) => println!("\n✗ Parse error: {}", e),
    }
    println!();
}

/// Validates user data structure
fn validate_user_structure(node: &Node) -> Result<(), String> {
    // Check if it's an object
    let obj = match node {
        Node::Mapping(map) => map,
        _ => return Err("Root must be an object".to_string()),
    };

    // Check required fields
    let required_fields = vec!["name", "age", "email"];
    for field in required_fields {
        if !obj.contains_key(field) {
            return Err(format!("Missing required field: {}", field));
        }
    }

    // Validate field types
    if let Some(name_node) = obj.get("name") {
        if !matches!(name_node, Node::String(_)) {
            return Err("Field 'name' must be a string".to_string());
        }
    }

    if let Some(age_node) = obj.get("age") {
        if !matches!(age_node, Node::Number(_)) {
            return Err("Field 'age' must be a number".to_string());
        }
    }

    if let Some(email_node) = obj.get("email") {
        if let Node::String(email) = email_node {
            if !email.contains('@') {
                return Err("Field 'email' must be a valid email address".to_string());
            }
        } else {
            return Err("Field 'email' must be a string".to_string());
        }
    }

    Ok(())
}

/// Demonstrates file error handling
fn demo_file_errors() {
    println!("--- Example 3: File Error Handling ---");

    // Try to open non-existent file
    println!("Attempting to open non-existent file:");
    match FileSource::new("nonexistent.yaml") {
        Ok(_) => println!("✓ File opened (unexpected)"),
        Err(e) => println!("✗ Expected error: {}", e),
    }

    // Create a valid file and test
    println!("\nAttempting to parse from valid source:");
    let yaml = "name: Test\nvalue: 123";
    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(_) => println!("✓ Parsed successfully"),
        Err(e) => println!("✗ Parse error: {}", e),
    }
    println!();
}

/// Demonstrates safe document access
fn demo_safe_document_access() {
    println!("--- Example 4: Safe Document Access ---");

    let yaml = r#"
---
document: first
---
document: second
---
document: third
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("✓ Parsed multi-document stream");

            // Safe access with error handling
            for i in 0..5 {
                print!("Accessing document {}: ", i);
                match get_document(&node, i) {
                    Ok(_doc) => println!("✓ Success"),
                    Err(e) => println!("✗ {}", e),
                }
            }
        }
        Err(e) => println!("✗ Parse error: {}", e),
    }
    println!();
}

/// Demonstrates defensive node traversal
fn demo_defensive_traversal() {
    println!("--- Example 5: Defensive Node Traversal ---");

    let yaml = r#"
user:
  name: John Doe
  profile:
    age: 30
    location:
      city: Springfield
      state: IL
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("✓ Parsed successfully");

            // Safe traversal with error handling
            println!("\nSafely accessing nested values:");

            // Try to get user.name
            let name = get_nested_string(&node, &["user", "name"]);
            println!("user.name: {}", name.unwrap_or("<not found>"));

            // Try to get user.profile.age
            let age = get_nested_number(&node, &["user", "profile", "age"]);
            match age {
                Some(n) => println!("user.profile.age: {}", n),
                None => println!("user.profile.age: <not found>"),
            }

            // Try to get non-existent path
            let invalid = get_nested_string(&node, &["user", "profile", "nonexistent"]);
            println!(
                "user.profile.nonexistent: {}",
                invalid.unwrap_or("<not found>")
            );

            // Try to get deeply nested value
            let city = get_nested_string(&node, &["user", "profile", "location", "city"]);
            println!(
                "user.profile.location.city: {}",
                city.unwrap_or("<not found>")
            );
        }
        Err(e) => println!("✗ Parse error: {}", e),
    }
    println!();
}

/// Safely gets a nested string value
fn get_nested_string<'a>(node: &'a Node, path: &[&str]) -> Option<&'a str> {
    let mut current = node;

    for (i, key) in path.iter().enumerate() {
        match current {
            Node::Mapping(map) => {
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

/// Safely gets a nested number value
fn get_nested_number(node: &Node, path: &[&str]) -> Option<i64> {
    let mut current = node;

    for key in path {
        match current {
            Node::Mapping(map) => {
                current = map.get(*key)?;
            }
            _ => return None,
        }
    }

    match current {
        Node::Number(yaml_lib::Numeric::Integer(n)) => Some(*n),
        _ => None,
    }
}

/// Demonstrates error recovery strategies
fn demo_error_recovery() {
    println!("--- Example 6: Error Recovery Strategies ---");

    let yaml_with_errors = r#"
# Some valid data
valid_field: value

# Some potentially problematic data
number_field: not_a_number
email_field: invalid-email-format

# More valid data
another_field: another_value
"#;

    let mut source = BufferSource::new(yaml_with_errors.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("✓ Parsed (YAML syntax is valid)");

            // Validate and provide defaults for invalid data
            println!("\nApplying error recovery:");

            if let Node::Mapping(map) = &node {
                // Check number field
                if let Some(num_node) = map.get("number_field") {
                    match num_node {
                        Node::Number(_) => println!("✓ number_field is valid"),
                        _ => println!("✗ number_field is not a number, using default: 0"),
                    }
                }

                // Check email field
                if let Some(email_node) = map.get("email_field") {
                    if let Node::String(email) = email_node {
                        if email.contains('@') {
                            println!("✓ email_field is valid");
                        } else {
                            println!("✗ email_field is invalid, using default: user@example.com");
                        }
                    }
                }

                // Successfully processed valid fields
                println!("✓ Other fields processed successfully");
            }
        }
        Err(e) => {
            println!("✗ Parse error: {}", e);
            println!("Recovery strategy: Using default configuration");
        }
    }
    println!();
}
