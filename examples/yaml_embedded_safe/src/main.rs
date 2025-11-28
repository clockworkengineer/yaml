//! Embedded-Safe YAML Example
//!
//! Demonstrates panic-free, safe YAML operations for embedded systems.
//! Shows best practices for resource-constrained environments.

extern crate alloc;

use alloc::vec::Vec;
use yaml_lib::embedded::limits::{LimitError, NodeValidator};
use yaml_lib::{BufferSource, Node, Numeric, parse};

fn main() {
    println!("=== Embedded-Safe YAML Example ===\n");

    // Example 1: Safe node access (no panics)
    safe_node_access_example();

    // Example 2: Numeric conversion for embedded
    numeric_conversion_example();

    // Example 3: Node validation against limits
    node_validation_example();

    // Example 4: Safe YAML parsing with validation
    safe_parsing_example();
}

/// Example 1: Safe node access without panicking
fn safe_node_access_example() {
    println!("--- Example 1: Safe Node Access ---");

    // Create a sample YAML structure
    let yaml = r#"
name: "Embedded System"
version: 1
ports: [8080, 8081, 8082]
config:
  timeout: 30
  retries: 3
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(doc_node) => {
            // Get the actual content from the document (unwrap nested documents)
            let mut node = &doc_node;
            loop {
                match node {
                    Node::Document(nodes) | Node::Documents(nodes) => {
                        if let Some(first) = nodes.first() {
                            node = first;
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }

            // Safe key access - returns Option instead of panicking
            if let Some(name) = node.get_key("name") {
                if let Some(name_str) = name.as_str() {
                    println!("Name (safe): {}", name_str);
                }
            }

            // Safe array access
            if let Some(ports) = node.get_key("ports") {
                if let Some(first_port) = ports.get(0) {
                    if let Some(port_num) = first_port.as_i32() {
                        println!("First port (safe): {}", port_num);
                    }
                }
            }

            // Check collection properties
            if let Some(config) = node.get_key("config") {
                if config.is_mapping() {
                    if let Some(len) = config.len() {
                        println!("Config has {} entries", len);
                    }
                }
            }

            // Safe access to non-existent key (no panic!)
            match node.get_key("nonexistent") {
                Some(_) => println!("Found nonexistent key"),
                None => println!("Key doesn't exist (handled safely)"),
            }
        }
        Err(e) => println!("Parse error: {:?}", e),
    }

    println!();
}

/// Example 2: Numeric conversion for embedded systems
fn numeric_conversion_example() {
    println!("--- Example 2: Numeric Conversion ---");

    // Create various numeric nodes
    let large_int = Node::Number(Numeric::Integer(1_000_000));
    let small_int = Node::Number(Numeric::Int32(42));
    let float_val = Node::Number(Numeric::Float(3.14159));

    // Convert to embedded-friendly i32
    match large_int.as_i32() {
        Some(val) => println!("Large int as i32: {}", val),
        None => println!("Large int doesn't fit in i32"),
    }

    match small_int.as_i32() {
        Some(val) => println!("Small int as i32: {}", val),
        None => println!("Small int conversion failed"),
    }

    // Convert to f32 for embedded systems
    if let Some(f32_val) = float_val.as_f32() {
        println!("Float as f32: {}", f32_val);
    }

    // Check numeric sizes
    let nums = vec![
        ("Integer(i64)", Numeric::Integer(0)),
        ("Int32(i32)", Numeric::Int32(0)),
        ("Int16(i16)", Numeric::Int16(0)),
        ("Byte(u8)", Numeric::Byte(0)),
    ];

    println!("\nNumeric type sizes:");
    for (name, num) in nums {
        println!("  {} = {} bytes", name, num.size_bytes());
    }

    // Check if values fit in i32 (recommended for embedded)
    let test_nums = vec![
        Numeric::Integer(1000),
        Numeric::Integer(i64::MAX),
        Numeric::Float(100.5),
    ];

    println!("\nFits in i32 check:");
    for num in test_nums {
        println!("  {:?} fits in i32: {}", num, num.fits_in_i32());
    }

    println!();
}

/// Example 3: Node validation against embedded limits
fn node_validation_example() {
    println!("--- Example 3: Node Validation ---");

    // Create a valid node structure
    let mut valid_items = Vec::new();
    for i in 0..10 {
        valid_items.push(Node::Number(Numeric::Int32(i)));
    }
    let valid_node = Node::Array(valid_items);

    let mut validator = NodeValidator::new();
    match validator.validate(&valid_node) {
        Ok(()) => {
            println!("✓ Valid node structure");
            println!("  Max depth: {}", validator.max_depth());
            println!("  Anchors: {}", validator.anchor_count());
        }
        Err(e) => println!("✗ Validation failed: {:?}", e),
    }

    // Create an overly nested structure (would exceed limits)
    println!("\nTesting deep nesting detection:");
    let mut deep_node = Node::None;
    for _ in 0..35 {
        deep_node = Node::Array(alloc::vec![deep_node]);
    }

    let mut validator = NodeValidator::new();
    match validator.validate(&deep_node) {
        Ok(()) => println!("✓ Deep nesting passed (unexpected)"),
        Err(LimitError::NestingDepthExceeded { current, max }) => {
            println!("✓ Correctly detected excessive nesting:");
            println!("  Current: {}, Max: {}", current, max);
        }
        Err(e) => println!("✗ Unexpected error: {:?}", e),
    }

    println!();
}

/// Example 4: Safe YAML parsing with validation
fn safe_parsing_example() {
    println!("--- Example 4: Safe Parsing with Validation ---");

    // Example of safe YAML that should pass validation
    let safe_yaml = r#"
device:
  name: "Sensor Node"
  id: 42
  readings:
    - temperature: 22
    - humidity: 65
    - pressure: 1013
"#;

    println!("Parsing safe YAML:");
    let mut source = BufferSource::new(safe_yaml.as_bytes());
    match parse(&mut source) {
        Ok(doc_node) => {
            // Get the actual content from the document (unwrap nested documents)
            let mut node = &doc_node;
            loop {
                match node {
                    Node::Document(nodes) | Node::Documents(nodes) => {
                        if let Some(first) = nodes.first() {
                            node = first;
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }

            let mut validator = NodeValidator::new();
            match validator.validate(&node) {
                Ok(()) => {
                    println!("✓ Parsed and validated successfully");
                    println!("  Max depth: {}", validator.max_depth());

                    // Safe data extraction
                    if let Some(device) = node.get_key("device") {
                        if let Some(name) = device.get_key("name") {
                            if let Some(name_str) = name.as_str() {
                                println!("  Device: {}", name_str);
                            }
                        }
                        if let Some(id) = device.get_key("id") {
                            if let Some(id_val) = id.as_i32() {
                                println!("  ID: {}", id_val);
                            }
                        }
                    }
                }
                Err(e) => println!("✗ Validation failed: {:?}", e),
            }
        }
        Err(e) => println!("✗ Parse error: {:?}", e),
    }

    // Example of YAML with very long string (would exceed limit)
    println!("\nTesting string length limit:");
    let long_string = "x".repeat(5000); // Exceeds MAX_STRING_LENGTH (4096)
    let yaml_with_long_string = format!("longstr: \"{}\"", long_string);

    let mut source = BufferSource::new(yaml_with_long_string.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            let mut validator = NodeValidator::new();
            match validator.validate(&node) {
                Ok(()) => println!("✓ Long string passed (unexpected)"),
                Err(LimitError::StringLengthExceeded { current, max }) => {
                    println!("✓ Correctly detected excessive string length:");
                    println!("  Current: {}, Max: {}", current, max);
                }
                Err(e) => println!("✗ Unexpected error: {:?}", e),
            }
        }
        Err(e) => println!("✗ Parse error: {:?}", e),
    }

    println!();
}
