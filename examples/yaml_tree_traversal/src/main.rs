//! Example demonstrating tree traversal and visitor pattern
//!
//! Shows how to efficiently navigate and transform YAML documents using
//! the new traversal APIs.

use yaml_lib::{parse, stringify, BufferSource, BufferDestination, Node, Numeric};

fn main() {
    println!("=== YAML Tree Traversal Examples ===\n");

    demo_children_iteration();
    demo_depth_first_traversal();
    demo_mutable_transformation();
    demo_searching();
    demo_filtering();
    demo_tree_statistics();
    demo_validation();
}

/// Demonstrates iterating over immediate children
fn demo_children_iteration() {
    println!("--- Example 1: Children Iteration ---");

    let yaml = r#"
users:
  - Alice
  - Bob
  - Charlie
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    if let Node::Documents(docs) = &doc {
        if let Node::Document(nodes) = &docs[0] {
            if let Node::Mapping(pairs) = &nodes[0] {
                let (_, users) = &pairs[0];

                println!("Iterating over array children:");
                for (i, child) in users.children().enumerate() {
                    if let Some(name) = child.as_str() {
                        println!("  Child {}: {}", i, name);
                    }
                }
                println!();
            }
        }
    }
}

/// Demonstrates depth-first traversal with visitor
fn demo_depth_first_traversal() {
    println!("--- Example 2: Depth-First Traversal ---");

    let yaml = r#"
config:
  server:
    host: localhost
    port: 8080
  database:
    host: db.example.com
    port: 5432
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    println!("Traversing document tree:");
    doc.visit(|node, depth| {
        let indent = "  ".repeat(depth);
        match node {
            Node::Str(s, _, _) => println!("{}String: \"{}\"", indent, s),
            Node::Number(n) => println!("{}Number: {:?}", indent, n),
            Node::Mapping(_) => println!("{}Mapping {{", indent),
            Node::Array(_) => println!("{}Array [", indent),
            _ => {}
        }
        true // Continue traversal
    });
    println!();
}

/// Demonstrates mutable transformation during traversal
fn demo_mutable_transformation() {
    println!("--- Example 3: Mutable Transformation ---");

    let yaml = r#"
prices:
  - 10
  - 20
  - 30
  - 40
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let mut doc = parse(&mut source).unwrap();

    let mut dest = BufferDestination::new();
    stringify(&doc, &mut dest).ok();
    println!("Original prices: {}", dest.to_string());

    // Double all numbers
    doc.visit_mut(|node, _depth| {
        if let Node::Number(Numeric::Int32(n)) = node {
            *n *= 2;
        }
        true
    });

    let mut dest2 = BufferDestination::new();
    stringify(&doc, &mut dest2).ok();
    println!("After doubling: {}\n", dest2.to_string());
}

/// Demonstrates searching with find_first and find_all
fn demo_searching() {
    println!("--- Example 4: Searching Nodes ---");

    let yaml = r#"
users:
  - name: Alice
    role: admin
    active: true
  - name: Bob
    role: user
    active: false
  - name: Charlie
    role: admin
    active: true
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    // Find first admin
    let first_admin = doc.find_first(|node| {
        if let Node::Str(s, _, _) = node {
            s == "admin"
        } else {
            false
        }
    });

    if first_admin.is_some() {
        println!("Found first admin node");
    }

    // Find all boolean nodes
    let booleans = doc.find_all(|node| node.is_boolean());
    println!("Found {} boolean nodes", booleans.len());

    // Find all mappings
    let mappings = doc.find_all(|node| node.is_mapping());
    println!("Found {} mappings\n", mappings.len());
}

/// Demonstrates filtering and collecting specific nodes
fn demo_filtering() {
    println!("--- Example 5: Filtering Nodes ---");

    let yaml = r#"
data:
  numbers: [1, 2, 3, 4, 5]
  text: "hello"
  enabled: true
  nested:
    value: 42
    name: "test"
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    // Find all string nodes
    let strings = doc.find_all(|node| node.is_str());
    println!("String nodes:");
    for s in &strings {
        if let Some(text) = s.as_str() {
            println!("  - \"{}\"", text);
        }
    }

    // Find all numeric nodes
    let numbers = doc.find_all(|node| node.is_number());
    println!("\nNumeric nodes:");
    for n in &numbers {
        if let Node::Number(num) = n {
            println!("  - {:?}", num);
        }
    }
    println!();
}

/// Demonstrates gathering tree statistics
fn demo_tree_statistics() {
    println!("--- Example 6: Tree Statistics ---");

    let yaml = r#"
application:
  name: MyApp
  version: 1.0.0
  features:
    - authentication
    - logging
    - caching
  config:
    database:
      host: localhost
      port: 5432
    server:
      host: 0.0.0.0
      port: 8080
      workers: 4
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    // Use built-in methods
    println!("Tree statistics:");
    println!("  Total nodes: {}", doc.count_nodes());
    println!("  Maximum depth: {}", doc.max_depth());

    // Count nodes by type
    let string_count = doc.find_all(|n| n.is_str()).len();
    let number_count = doc.find_all(|n| n.is_number()).len();
    let array_count = doc.find_all(|n| n.is_array()).len();
    let mapping_count = doc.find_all(|n| n.is_mapping()).len();

    println!("  Strings: {}", string_count);
    println!("  Numbers: {}", number_count);
    println!("  Arrays: {}", array_count);
    println!("  Mappings: {}\n", mapping_count);
}

/// Demonstrates custom validation using traversal
fn demo_validation() {
    println!("--- Example 7: Custom Validation ---");

    let yaml = r#"
config:
  timeout: 30
  retries: 3
  endpoints:
    - url: "https://api1.example.com"
      enabled: true
    - url: "https://api2.example.com"
      enabled: false
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    // Validate: check if any timeouts exceed limit
    let mut valid = true;
    let max_timeout = 60;

    doc.visit(|node, _depth| {
        // Check if this is a "timeout" value that's too high
        if let Node::Number(Numeric::Int32(n)) = node {
            if *n > max_timeout {
                println!(
                    "⚠ Warning: timeout value {} exceeds maximum {}",
                    n, max_timeout
                );
                valid = false;
            }
        }
        true // Continue checking
    });

    // Validate: ensure all URLs are HTTPS
    doc.visit(|node, _depth| {
        if let Node::Str(s, _, _) = node {
            if s.starts_with("http://") {
                println!("⚠ Warning: insecure HTTP URL found: {}", s);
                valid = false;
            }
        }
        true
    });

    if valid {
        println!("✓ All validations passed!");
    }
    println!();
}
