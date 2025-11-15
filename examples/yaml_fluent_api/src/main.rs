//! Example demonstrating the fluent API for building YAML nodes
//!
//! This example shows how to use the new builder pattern for constructing
//! complex YAML structures with a clean, readable syntax.

use yaml_lib::{stringify, BufferDestination, Node};

fn main() {
    println!("=== YAML Fluent API Example ===\n");

    // Example 1: Simple array construction
    demo_simple_array();

    // Example 2: Simple mapping construction
    demo_simple_mapping();

    // Example 3: Nested structures
    demo_nested_structures();

    // Example 4: Conditional building
    demo_conditional_building();

    // Example 5: Complex application configuration
    demo_complex_config();

    // Example 6: Set construction with automatic deduplication
    demo_set_building();
}

/// Demonstrates simple array construction
fn demo_simple_array() {
    println!("--- Example 1: Simple Array ---");

    // Old verbose way
    let old_way = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);

    // New fluent way
    let new_way = Node::array().push(1).push(2).push(3).build();

    println!("Old way and new way are equivalent: {}", old_way == new_way);

    let mut dest = BufferDestination::new();
    stringify(&new_way, &mut dest).unwrap();
    println!("Result:\n{}\n", dest.to_string());
}

/// Demonstrates simple mapping construction
fn demo_simple_mapping() {
    println!("--- Example 2: Simple Mapping ---");

    let person = Node::mapping()
        .insert("name", "Alice")
        .insert("age", 30)
        .insert("email", "alice@example.com")
        .insert("active", true)
        .build();

    let mut dest = BufferDestination::new();
    stringify(&person, &mut dest).unwrap();
    println!("Person:\n{}\n", dest.to_string());
}

/// Demonstrates nested structures
fn demo_nested_structures() {
    println!("--- Example 3: Nested Structures ---");

    let document = Node::mapping()
        .insert("title", "Product Catalog")
        .insert(
            "products",
            Node::array()
                .push(
                    Node::mapping()
                        .insert("id", 1)
                        .insert("name", "Widget")
                        .insert("price", 19.99)
                        .build(),
                )
                .push(
                    Node::mapping()
                        .insert("id", 2)
                        .insert("name", "Gadget")
                        .insert("price", 29.99)
                        .build(),
                )
                .build(),
        )
        .build();

    let mut dest = BufferDestination::new();
    stringify(&document, &mut dest).unwrap();
    println!("Catalog:\n{}\n", dest.to_string());
}

/// Demonstrates conditional building
fn demo_conditional_building() {
    println!("--- Example 4: Conditional Building ---");

    let debug_mode = true;
    let optional_feature: Option<&str> = Some("premium");
    let disabled_feature: Option<&str> = None;

    let config = Node::mapping()
        .insert("app_name", "MyApp")
        .insert("version", "1.0.0")
        // Only include if debug_mode is true
        .insert_if(debug_mode, "debug_logs", true)
        .insert_if(debug_mode, "verbose", true)
        // Only include if option is Some
        .insert_opt("premium_feature", optional_feature)
        .insert_opt("disabled_feature", disabled_feature)
        .build();

    let mut dest = BufferDestination::new();
    stringify(&config, &mut dest).unwrap();
    println!("Conditional config:\n{}\n", dest.to_string());
}

/// Demonstrates complex application configuration
fn demo_complex_config() {
    println!("--- Example 5: Complex Configuration ---");

    let config = Node::mapping()
        .insert(
            "application",
            Node::mapping()
                .insert("name", "WebAPI")
                .insert("version", "2.1.0")
                .insert("environment", "production")
                .insert("description", "RESTful API Server")
                .build(),
        )
        .insert(
            "server",
            Node::mapping()
                .insert("host", "0.0.0.0")
                .insert("port", 8080)
                .insert("workers", 4)
                .insert("timeout", 30)
                .build(),
        )
        .insert(
            "database",
            Node::mapping()
                .insert("host", "localhost")
                .insert("port", 5432)
                .insert("name", "app_db")
                .insert("ssl", true)
                .insert(
                    "pool",
                    Node::mapping()
                        .insert("min_connections", 5)
                        .insert("max_connections", 20)
                        .build(),
                )
                .build(),
        )
        .insert(
            "features",
            Node::array()
                .push("authentication")
                .push("authorization")
                .push("logging")
                .push("metrics")
                .push("caching")
                .build(),
        )
        .insert(
            "endpoints",
            Node::array()
                .push("/api/v1/users")
                .push("/api/v1/products")
                .push("/api/v1/orders")
                .build(),
        )
        .insert(
            "security",
            Node::mapping()
                .insert("cors_enabled", true)
                .insert("rate_limiting", true)
                .insert(
                    "allowed_origins",
                    Node::array()
                        .push("https://example.com")
                        .push("https://app.example.com")
                        .build(),
                )
                .build(),
        )
        .build();

    let mut dest = BufferDestination::new();
    stringify(&config, &mut dest).unwrap();
    println!("Complex configuration:\n{}\n", dest.to_string());
}

/// Demonstrates set construction with automatic deduplication
fn demo_set_building() {
    println!("--- Example 6: Set Building ---");

    let tags = Node::set()
        .insert("rust")
        .insert("yaml")
        .insert("parser")
        .insert("rust") // duplicate, will be ignored
        .insert("library")
        .insert("yaml") // duplicate, will be ignored
        .build();

    println!("Set (duplicates removed automatically):");
    let mut dest = BufferDestination::new();
    stringify(&tags, &mut dest).unwrap();
    println!("{}\n", dest.to_string());

    // Using extend with duplicates
    let more_tags = Node::set()
        .extend(vec!["web", "api", "server", "web", "api"]) // has duplicates
        .build();

    println!("Set from extend (duplicates removed):");
    let mut dest = BufferDestination::new();
    stringify(&more_tags, &mut dest).unwrap();
    println!("{}\n", dest.to_string());
}
