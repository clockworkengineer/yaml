//! Example demonstrating programmatic Node creation and manipulation
//!
//! This example shows how to:
//! - Create nodes programmatically using constructors
//! - Use the make_node() function for convenient creation
//! - Manipulate existing node structures
//! - Work with different node types

use yaml_lib::{make_node, make_set, stringify, BufferDestination, Node};

/// Helper function to print node as YAML
fn print_node(node: &Node) {
    let mut dest = BufferDestination::new();
    match stringify(node, &mut dest) {
        Ok(_) => println!("{}", dest.to_string().trim()),
        Err(e) => eprintln!("Stringify error: {}", e),
    }
}

fn main() {
    println!("=== YAML Node Manipulation Example ===\n");

    // Example 1: Creating basic nodes
    demo_basic_nodes();

    // Example 2: Creating arrays
    demo_arrays();

    // Example 3: Creating mappings
    demo_mappings();

    // Example 4: Creating sets
    demo_sets();

    // Example 5: Node manipulation
    demo_manipulation();
}

/// Demonstrates basic node creation
fn demo_basic_nodes() {
    println!("--- Example 1: Basic Node Creation ---");

    // Using make_node() function for simple values
    let name = make_node("Alice");
    let age = make_node(25);
    let score = make_node(95.5);
    let active = make_node(true);

    println!("Simple scalar nodes:");
    print_node(&name);
    print_node(&age);
    print_node(&score);
    print_node(&active);
    println!();
}

/// Demonstrates array creation
fn demo_arrays() {
    println!("--- Example 2: Creating Arrays ---");

    // Create an array using Vec
    let colors = Node::Array(vec![
        make_node("red"),
        make_node("green"),
        make_node("blue"),
    ]);

    println!("Array of strings:");
    print_node(&colors);

    // Array of numbers
    let numbers = Node::Array(vec![
        make_node(1),
        make_node(2),
        make_node(3),
        make_node(4),
        make_node(5),
    ]);

    println!("\nArray of numbers:");
    print_node(&numbers);
    println!();
}

/// Demonstrates mapping (object) creation
fn demo_mappings() {
    println!("--- Example 3: Creating Mappings ---");

    // Create a simple mapping using Node::Mapping
    let person = Node::Mapping(vec![
        (make_node("name"), make_node("Bob Smith")),
        (make_node("age"), make_node(35)),
        (make_node("email"), make_node("bob@example.com")),
    ]);

    println!("Person mapping:");
    print_node(&person);

    // Nested mapping
    let config = Node::Mapping(vec![
        (
            make_node("database"),
            Node::Mapping(vec![
                (make_node("host"), make_node("localhost")),
                (make_node("port"), make_node(5432)),
            ]),
        ),
        (
            make_node("servers"),
            Node::Array(vec![
                make_node("web1"),
                make_node("web2"),
                make_node("web3"),
            ]),
        ),
        (make_node("debug"), make_node(true)),
    ]);

    println!("\nNested configuration:");
    print_node(&config);
    println!();
}

/// Demonstrates set creation
fn demo_sets() {
    println!("--- Example 4: Creating Sets ---");

    // Create a set with duplicates - duplicates will be removed
    let tags = make_set(vec![
        make_node("rust"),
        make_node("yaml"),
        make_node("parsing"),
        make_node("rust"), // Duplicate - will be removed
        make_node("yaml"), // Duplicate - will be removed
        make_node("serialization"),
    ]);

    println!("Set with automatic duplicate removal:");
    print_node(&tags);

    // Another set example
    let numbers = make_set(vec![
        make_node(1),
        make_node(2),
        make_node(3),
        make_node(2), // Duplicate
        make_node(4),
        make_node(1), // Duplicate
        make_node(5),
    ]);

    println!("\nNumeric set:");
    print_node(&numbers);
    println!();
}

/// Demonstrates node manipulation
fn demo_manipulation() {
    println!("--- Example 5: Node Manipulation ---");

    // Create an initial array
    let mut features = Node::Array(vec![make_node("basic"), make_node("standard")]);

    println!("Original features:");
    print_node(&features);

    // Add items to the array
    if let Node::Array(ref mut arr) = features {
        arr.push(make_node("advanced"));
        arr.push(make_node("premium"));
    }

    println!("\nModified features:");
    print_node(&features);

    // Create and manipulate a mapping
    let mut config = Node::Mapping(vec![
        (make_node("version"), make_node("1.0.0")),
        (make_node("name"), make_node("MyApp")),
    ]);

    println!("\nOriginal config:");
    print_node(&config);

    // Update the mapping by adding more key-value pairs
    if let Node::Mapping(ref mut pairs) = config {
        pairs.push((make_node("build_date"), make_node("2024-01-15")));
        pairs.push((make_node("author"), make_node("Development Team")));
    }

    println!("\nUpdated config:");
    print_node(&config);
    println!();
}
