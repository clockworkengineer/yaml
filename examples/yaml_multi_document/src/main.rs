//! Example demonstrating YAML multi-document streams
//!
//! This example shows how to:
//! - Parse YAML files containing multiple documents
//! - Access individual documents from a stream
//! - Count documents in a stream
//! - Work with document separators (---)
//! - Handle end-of-document markers (...)

use yaml_lib::{
    get_document, get_number_of_documents, parse, stringify, BufferDestination, BufferSource, Node,
};

fn main() {
    println!("=== YAML Multi-Document Streams Example ===\n");

    // Example 1: Basic multi-document parsing
    demo_basic_multi_doc();

    // Example 2: Accessing individual documents
    demo_document_access();

    // Example 3: Configuration per environment
    demo_environment_configs();

    // Example 4: Data batches
    demo_data_batches();

    // Example 5: Mixed document types
    demo_mixed_documents();
}

/// Demonstrates basic multi-document parsing
fn demo_basic_multi_doc() {
    println!("--- Example 1: Basic Multi-Document Stream ---");

    let yaml = r#"
---
document: first
name: Document 1
data:
  - item1
  - item2
---
document: second
name: Document 2
data:
  - item3
  - item4
---
document: third
name: Document 3
data:
  - item5
  - item6
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            match get_number_of_documents(&node) {
                Ok(count) => {
                    println!("Parsed {} documents", count);

                    // Print all documents
                    for i in 0..count {
                        println!("\n--- Document {} ---", i);
                        match get_document(&node, i) {
                            Ok(doc) => print_node(&doc),
                            Err(e) => eprintln!("Error accessing document {}: {}", i, e),
                        }
                    }
                }
                Err(e) => eprintln!("Error counting documents: {}", e),
            }
            println!();
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Demonstrates accessing individual documents
fn demo_document_access() {
    println!("--- Example 2: Accessing Individual Documents ---");

    let yaml = r#"
---
# First document: user data
user:
  id: 1001
  name: John Doe
  email: john@example.com
---
# Second document: user preferences
preferences:
  theme: dark
  language: en
  notifications: true
---
# Third document: user activity
activity:
  last_login: 2024-01-15
  login_count: 42
  active: true
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            match get_number_of_documents(&node) {
                Ok(count) => println!("Total documents: {}", count),
                Err(e) => eprintln!("Error counting documents: {}", e),
            };

            // Access specific documents
            println!("\nAccessing Document 0 (User Data):");
            if let Ok(doc) = get_document(&node, 0) {
                print_node(&doc);
            }

            println!("\nAccessing Document 1 (Preferences):");
            if let Ok(doc) = get_document(&node, 1) {
                print_node(&doc);
            }

            println!("\nAccessing Document 2 (Activity):");
            if let Ok(doc) = get_document(&node, 2) {
                print_node(&doc);
            }

            // Try to access non-existent document
            println!("\nTrying to access Document 3 (doesn't exist):");
            match get_document(&node, 3) {
                Ok(_) => println!("Unexpected success!"),
                Err(e) => println!("Expected error: {}", e),
            }
            println!();
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Demonstrates environment-specific configurations
fn demo_environment_configs() {
    println!("--- Example 3: Environment Configurations ---");

    let yaml = r#"
---
# Development environment
environment: development
database:
  host: localhost
  port: 5432
  name: myapp_dev
  debug: true
server:
  port: 3000
  hot_reload: true
logging:
  level: debug
---
# Staging environment
environment: staging
database:
  host: staging-db.example.com
  port: 5432
  name: myapp_staging
  debug: false
server:
  port: 8080
  hot_reload: false
logging:
  level: info
---
# Production environment
environment: production
database:
  host: prod-db.example.com
  port: 5432
  name: myapp_prod
  debug: false
  ssl: true
server:
  port: 443
  ssl: true
  hot_reload: false
logging:
  level: error
  aggregation: true
...
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            let count = get_number_of_documents(&node);
            match count {
                Ok(count) => println!("Loaded {} environment configurations", count),
                Err(e) => eprintln!("Error counting documents: {}", e),
            }

            let environments = vec!["development", "staging", "production"];
            for (i, env) in environments.iter().enumerate() {
                println!("\n{} Configuration:", env);
                if let Ok(doc) = get_document(&node, i) {
                    print_node(&doc);
                }
            }
            println!();
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Demonstrates data batches in documents
fn demo_data_batches() {
    println!("--- Example 4: Data Batches ---");

    let yaml = r#"
---
# Batch 1: User records
batch_id: 1
timestamp: 2024-01-15T10:00:00Z
records:
  - id: 1001
    name: Alice Smith
    status: active
  - id: 1002
    name: Bob Jones
    status: active
---
# Batch 2: User records
batch_id: 2
timestamp: 2024-01-15T10:05:00Z
records:
  - id: 1003
    name: Charlie Brown
    status: pending
  - id: 1004
    name: Diana Prince
    status: active
---
# Batch 3: User records
batch_id: 3
timestamp: 2024-01-15T10:10:00Z
records:
  - id: 1005
    name: Eve Wilson
    status: inactive
  - id: 1006
    name: Frank Miller
    status: active
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            let doc_count = get_number_of_documents(&node);
            match doc_count {
                Ok(num) => {
                    println!("Processing {} data batches\n", num);

                    for i in 0..num {
                        println!("Batch {}:", i + 1);
                        if let Ok(doc) = get_document(&node, i) {
                            print_node(&doc);
                        }
                        println!();
                    }
                }
                Err(e) => eprintln!("Error counting documents: {}", e),
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Demonstrates mixed document types
fn demo_mixed_documents() {
    println!("--- Example 5: Mixed Document Types ---");

    let yaml = r#"
---
# Document 1: Mapping
type: configuration
name: Main Config
settings:
  timeout: 30
  retries: 3
---
# Document 2: Sequence
- id: 1
  name: First Item
- id: 2
  name: Second Item
- id: 3
  name: Third Item
---
# Document 3: Scalar
"This is a simple string document"
---
# Document 4: Nested structure
type: complex
data:
  arrays:
    - [1, 2, 3]
    - [4, 5, 6]
  mappings:
    key1:
      nested1: value1
    key2:
      nested2: value2
---
# Document 5: Empty document (null)
...
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            let doc_count = get_number_of_documents(&node);
            match &doc_count {
                Ok(count) => println!("Parsed {} documents of different types\n", count),
                Err(e) => eprintln!("Error counting documents: {}", e),
            }

            if let Ok(num) = doc_count {
                for i in 0..num {
                    match get_document(&node, i) {
                        Ok(doc) => {
                            let doc_type = match &doc {
                                Node::Mapping(_) => "Mapping",
                                Node::Array(_) => "Sequence",
                                Node::Str(_, _, _) => "String",
                                Node::Number(_) => "Number",
                                Node::Boolean(_) => "Boolean",
                                Node::None => "Null",
                                _ => "Other",
                            };
                            println!("Document {} - Type: {}", i, doc_type);
                            print_node(&doc);
                            println!();
                        }
                        Err(e) => eprintln!("Error accessing document {}: {}", i, e),
                    }
                }
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Helper function to print node structure
fn print_node(node: &Node) {
    let mut dest = BufferDestination::new();
    match stringify(node, &mut dest) {
        Ok(_) => {
            let output = dest.to_string();
            // Print first few lines for brevity
            for line in output.lines().take(15) {
                println!("{}", line);
            }
        }
        Err(e) => eprintln!("Stringify error: {}", e),
    }
}
