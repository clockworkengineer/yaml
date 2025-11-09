//! Example demonstrating YAML anchors and aliases for node reuse
//!
//! This example shows how to:
//! - Parse YAML with anchors (&) and aliases (*)
//! - Reuse configuration blocks across multiple sections
//! - Understand how the library resolves aliases
//! - Work with the !!merge tag for inheritance patterns

use yaml_lib::{parse, stringify, BufferDestination, BufferSource, Node};

fn main() {
    println!("=== YAML Anchors and Aliases Example ===\n");

    // Example 1: Basic anchors and aliases
    demo_basic_anchors();

    // Example 2: Configuration inheritance with merge keys
    demo_merge_keys();

    // Example 3: Complex nested anchors
    demo_nested_anchors();

    // Example 4: Multiple references to same anchor
    demo_multiple_references();
}

/// Demonstrates basic anchor and alias usage
fn demo_basic_anchors() {
    println!("--- Example 1: Basic Anchors and Aliases ---");

    let yaml = r#"
# Define a default configuration with an anchor
default_config: &defaults
  timeout: 30
  retries: 3
  log_level: info

# Reference the anchor using an alias
service_a: *defaults

service_b: *defaults
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("Parsed YAML with anchors:");
            print_node(&node);
            println!();
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Demonstrates merge keys for configuration inheritance
fn demo_merge_keys() {
    println!("--- Example 2: Configuration Inheritance with Merge Keys ---");

    let yaml = r#"
# Base configuration
base: &base_config
  timeout: 30
  retries: 3
  log_level: info
  cache_enabled: true

# Development environment - inherits from base and overrides
development:
  <<: *base_config
  log_level: debug
  debug_mode: true

# Production environment - inherits from base with different overrides
production:
  <<: *base_config
  timeout: 60
  log_level: warn
  cache_enabled: true
  monitoring: true

# Staging environment
staging:
  <<: *base_config
  log_level: info
  monitoring: true
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("Configuration with merge keys:");
            print_node(&node);

            // Demonstrate round-trip
            let mut dest = BufferDestination::new();
            if stringify(&node, &mut dest).is_ok() {
                println!("\nStringified back to YAML:");
                println!("{}", dest.to_string());
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Demonstrates nested anchors and complex references
fn demo_nested_anchors() {
    println!("--- Example 3: Nested Anchors ---");

    let yaml = r#"
# Nested structure with anchor
database_config: &db
  host: localhost
  port: 5432
  credentials: &creds
    username: admin
    password: secret
  pool:
    min_size: 5
    max_size: 20

# Reuse entire database config
primary_db: *db

# Reuse just the credentials
backup_db:
  host: backup.example.com
  port: 5432
  credentials: *creds
  pool:
    min_size: 2
    max_size: 10
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("Nested anchors and references:");
            print_node(&node);
            println!();
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Demonstrates multiple references to the same anchor
fn demo_multiple_references() {
    println!("--- Example 4: Multiple References ---");

    let yaml = r#"
# A list that will be referenced multiple times
common_features: &features
  - authentication
  - logging
  - metrics
  - health_check

# Multiple services using the same feature list
services:
  web_api:
    name: Web API
    port: 8080
    features: *features
  
  admin_api:
    name: Admin API
    port: 8081
    features: *features
  
  metrics_api:
    name: Metrics API
    port: 9090
    features: *features

# Also use in a different context
monitoring:
  required_features: *features
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("Multiple references to same anchor:");
            print_node(&node);
            println!();
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Helper function to print node structure
fn print_node(node: &Node) {
    let mut dest = BufferDestination::new();
    match stringify(node, &mut dest) {
        Ok(_) => println!("{}", dest.to_string()),
        Err(e) => eprintln!("Stringify error: {}", e),
    }
}
