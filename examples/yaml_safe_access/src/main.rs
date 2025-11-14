//! Example demonstrating safe, panic-free node access API
//!
//! This example shows how to safely access YAML nodes without risking panics,
//! using the new safe access methods introduced in the library.

use yaml_lib::{parse, BufferSource, Node};

fn main() {
    println!("=== YAML Safe Access API Examples ===\n");

    demo_safe_array_access();
    demo_safe_mapping_access();
    demo_type_conversions();
    demo_nested_access();
    demo_collection_views();
    demo_config_loading();
}

/// Demonstrates safe array/sequence access
fn demo_safe_array_access() {
    println!("--- Example 1: Safe Array Access ---");

    let yaml = r#"
numbers: [10, 20, 30, 40, 50]
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    if let Node::Documents(docs) = &doc {
        if let Node::Document(nodes) = &docs[0] {
            if let Node::Mapping(pairs) = &nodes[0] {
                let (_, array) = &pairs[0];

                // Safe access - returns Option
                println!("Third element: {:?}", array.get(2)); // Some(30)
                println!("Out of bounds: {:?}", array.get(99)); // None

                // Check before accessing
                if let Some(value) = array.get(1) {
                    println!("Second element exists: {:?}", value);
                }

                // Use with unwrap_or for defaults
                let default_node = Node::from(0);
                let first = array.get(0).unwrap_or(&default_node);
                println!("First element (with default): {:?}\n", first);
            }
        }
    }
}

/// Demonstrates safe mapping/object key access
fn demo_safe_mapping_access() {
    println!("--- Example 2: Safe Mapping Access ---");

    let yaml = r#"
database:
  host: localhost
  port: 5432
  name: myapp
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    if let Node::Documents(docs) = &doc {
        if let Node::Document(nodes) = &docs[0] {
            if let Node::Mapping(pairs) = &nodes[0] {
                let (_, db_config) = &pairs[0];

                // Safe key access
                println!("Host: {:?}", db_config.get_key("host"));
                println!("Missing key: {:?}", db_config.get_key("nonexistent"));

                // Check if key exists
                if db_config.contains_key("port") {
                    println!("Port configuration is present");
                }

                // Get all keys
                let keys = db_config.keys();
                println!("Configuration keys: {:?}\n", keys);
            }
        }
    }
}

/// Demonstrates type conversion methods
fn demo_type_conversions() {
    println!("--- Example 3: Type Conversions ---");

    let yaml = r#"
string_value: "hello"
number_value: 42
float_value: 3.14
bool_value: true
null_value: null
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    if let Node::Documents(docs) = &doc {
        if let Node::Document(nodes) = &docs[0] {
            if let Node::Mapping(pairs) = &nodes[0] {
                for (key, value) in pairs {
                    if let Some(key_str) = key.as_str() {
                        println!("Key: {}", key_str);

                        // Try different type conversions
                        if let Some(s) = value.as_str() {
                            println!("  -> String: {}", s);
                        } else if let Some(i) = value.as_i32() {
                            println!("  -> i32: {}", i);
                        } else if let Some(f) = value.as_f32() {
                            println!("  -> f32: {}", f);
                        } else if let Some(b) = value.as_bool() {
                            println!("  -> bool: {}", b);
                        } else if value.is_none() {
                            println!("  -> null");
                        }
                    }
                }
                println!();
            }
        }
    }
}

/// Demonstrates nested safe access patterns
fn demo_nested_access() {
    println!("--- Example 4: Nested Access Patterns ---");

    let yaml = r#"
server:
  database:
    primary:
      host: db1.example.com
      port: 5432
    replica:
      host: db2.example.com
      port: 5433
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    if let Node::Documents(docs) = &doc {
        if let Node::Document(nodes) = &docs[0] {
            if let Node::Mapping(pairs) = &nodes[0] {
                let (_, root) = &pairs[0];

                // Chained safe access
                let primary_host = root
                    .get_key("database")
                    .and_then(|db| db.get_key("primary"))
                    .and_then(|primary| primary.get_key("host"))
                    .and_then(|host| host.as_str())
                    .unwrap_or("unknown");

                println!("Primary DB host: {}", primary_host);

                // Access with default fallback
                let replica_port = root
                    .get_key("database")
                    .and_then(|db| db.get_key("replica"))
                    .and_then(|replica| replica.get_key("port"))
                    .and_then(|port| port.as_i32())
                    .unwrap_or(5432);

                println!("Replica DB port: {}\n", replica_port);
            }
        }
    }
}

/// Demonstrates collection view methods
fn demo_collection_views() {
    println!("--- Example 5: Collection Views ---");

    let yaml = r#"
tags: [rust, yaml, parsing, safe]
metadata:
  author: developer
  version: 1.0
  license: MIT
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    if let Node::Documents(docs) = &doc {
        if let Node::Document(nodes) = &docs[0] {
            if let Node::Mapping(pairs) = &nodes[0] {
                // Array as slice
                let (_, tags) = &pairs[0];
                if let Some(slice) = tags.as_slice() {
                    println!("Tags count: {}", slice.len());
                    for tag in slice {
                        if let Some(s) = tag.as_str() {
                            println!("  - {}", s);
                        }
                    }
                }

                // Mapping as slice of pairs
                let (_, metadata) = &pairs[1];
                if let Some(mapping) = metadata.as_mapping() {
                    println!("\nMetadata entries: {}", mapping.len());
                    for (key, value) in mapping {
                        if let (Some(k), Some(v)) = (key.as_str(), value.as_str()) {
                            println!("  {}: {}", k, v);
                        }
                    }
                }
                println!();
            }
        }
    }
}

/// Demonstrates practical config loading with safe access
fn demo_config_loading() {
    println!("--- Example 6: Practical Configuration Loading ---");

    let yaml = r#"
app:
  name: MyApplication
  debug: true
  server:
    host: 0.0.0.0
    # port is missing - will use default
  database:
    host: localhost
    port: 5432
    pool_size: 10
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    // Safe configuration extraction with defaults
    struct AppConfig {
        name: String,
        debug: bool,
        server_host: String,
        server_port: i32,
        db_host: String,
        db_port: i32,
        db_pool_size: i32,
    }

    if let Node::Documents(docs) = &doc {
        if let Node::Document(nodes) = &docs[0] {
            if let Node::Mapping(pairs) = &nodes[0] {
                let (_, app_node) = &pairs[0];

                let config = AppConfig {
                    name: app_node
                        .get_key("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("DefaultApp")
                        .to_string(),

                    debug: app_node
                        .get_key("debug")
                        .and_then(|d| d.as_bool())
                        .unwrap_or(false),

                    server_host: app_node
                        .get_key("server")
                        .and_then(|s| s.get_key("host"))
                        .and_then(|h| h.as_str())
                        .unwrap_or("localhost")
                        .to_string(),

                    server_port: app_node
                        .get_key("server")
                        .and_then(|s| s.get_key("port"))
                        .and_then(|p| p.as_i32())
                        .unwrap_or(8080), // Default because missing

                    db_host: app_node
                        .get_key("database")
                        .and_then(|d| d.get_key("host"))
                        .and_then(|h| h.as_str())
                        .unwrap_or("localhost")
                        .to_string(),

                    db_port: app_node
                        .get_key("database")
                        .and_then(|d| d.get_key("port"))
                        .and_then(|p| p.as_i32())
                        .unwrap_or(5432),

                    db_pool_size: app_node
                        .get_key("database")
                        .and_then(|d| d.get_key("pool_size"))
                        .and_then(|p| p.as_i32())
                        .unwrap_or(5),
                };

                println!("Loaded configuration:");
                println!("  App Name: {}", config.name);
                println!("  Debug Mode: {}", config.debug);
                println!("  Server: {}:{}", config.server_host, config.server_port);
                println!(
                    "  Database: {}:{} (pool: {})",
                    config.db_host, config.db_port, config.db_pool_size
                );
                println!("\nNote: server_port used default (8080) because it was missing!");
            }
        }
    }
}
