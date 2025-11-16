//! Example demonstrating streaming and iterator support for YAML processing
//!
//! This example shows how to:
//! - Iterate through node trees with depth-first and breadth-first traversal
//! - Use filter/map/fold operations on node streams
//! - Access nested nodes using paths
//! - Process large documents efficiently
//! - Collect specific node types

use yaml_lib::{parse, BufferSource, Node, NodeIteratorExt, NodePath, NodeStream, Numeric, PathSegment};

fn main() {
    println!("=== YAML Streaming and Iterator Example ===\n");

    // Example 1: Basic iteration
    demo_basic_iteration();

    // Example 2: Traversal orders
    demo_traversal_orders();

    // Example 3: Filtering and searching
    demo_filtering();

    // Example 4: Stream operations (map/fold)
    demo_stream_operations();

    // Example 5: Path-based access
    demo_path_access();

    // Example 6: Collecting specific types
    demo_collecting();

    // Example 7: Processing large documents
    demo_large_document();
}

/// Demonstrates basic node iteration
fn demo_basic_iteration() {
    println!("--- Example 1: Basic Iteration ---");

    let yaml = r#"
name: Alice
age: 30
hobbies:
  - reading
  - coding
  - hiking
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("Iterating through all nodes:");
            let mut count = 0;
            for n in node.iter_depth_first() {
                count += 1;
                match n {
                    Node::Str(s, _, _) => println!("  String: {}", s),
                    Node::Number(num) => println!("  Number: {:?}", num),
                    Node::Array(_) => println!("  Array"),
                    Node::Mapping(_) => println!("  Mapping"),
                    _ => {}
                }
            }
            println!("\nTotal nodes: {}", count);
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
    println!();
}

/// Demonstrates depth-first vs breadth-first traversal
fn demo_traversal_orders() {
    println!("--- Example 2: Traversal Orders ---");

    let tree = Node::Array(vec![
        Node::from("level1-1"),
        Node::Array(vec![Node::from("level2-1"), Node::from("level2-2")]),
        Node::from("level1-2"),
    ]);

    println!("Depth-first traversal:");
    for (i, node) in tree.iter_depth_first().enumerate() {
        if let Node::Str(s, _, _) = node {
            println!("  {}: {}", i, s);
        }
    }

    println!("\nBreadth-first traversal:");
    for (i, node) in tree.iter_breadth_first().enumerate() {
        if let Node::Str(s, _, _) = node {
            println!("  {}: {}", i, s);
        }
    }
    println!();
}

/// Demonstrates filtering and searching nodes
fn demo_filtering() {
    println!("--- Example 3: Filtering and Searching ---");

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
    match parse(&mut source) {
        Ok(node) => {
            // Find first "admin" string
            println!("Finding first 'admin':");
            if let Some(found) = node.find_node(|n| {
                matches!(n, Node::Str(s, _, _) if s == "admin")
            }) {
                println!("  Found: {:?}", found);
            }

            // Filter all "admin" strings
            println!("\nFiltering all 'admin' roles:");
            let admins = node.filter_nodes(|n| {
                matches!(n, Node::Str(s, _, _) if s == "admin")
            });
            println!("  Found {} admin roles", admins.len());

            // Count specific node types
            let string_count = node.filter_nodes(|n| matches!(n, Node::Str(_, _, _))).len();
            let array_count = node.filter_nodes(|n| matches!(n, Node::Array(_))).len();
            let mapping_count = node.filter_nodes(|n| matches!(n, Node::Mapping(_))).len();
            
            println!("\nNode type counts:");
            println!("  Strings: {}", string_count);
            println!("  Arrays: {}", array_count);
            println!("  Mappings: {}", mapping_count);
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
    println!();
}

/// Demonstrates stream operations (filter, map, fold)
fn demo_stream_operations() {
    println!("--- Example 4: Stream Operations ---");

    let yaml = r#"
prices:
  - 10.50
  - 25.00
  - 7.99
  - 42.00
  - 15.75
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            // Filter: Find prices over $20
            println!("Prices over $20:");
            let stream = NodeStream::new(&node);
            let expensive: Vec<_> = stream
                .filter(|n| {
                    matches!(n, Node::Number(Numeric::Float(price)) if *price > 20.0)
                })
                .collect();
            println!("  Found {} expensive items", expensive.len());

            // Map: Extract all prices
            println!("\nAll prices:");
            let stream = NodeStream::new(&node);
            let prices: Vec<_> = stream
                .map(|n| match n {
                    Node::Number(Numeric::Float(price)) => Some(*price),
                    _ => None,
                })
                .filter_map(|x| x)
                .collect();
            println!("  {:?}", prices);

            // Fold: Calculate total
            println!("\nCalculating total:");
            let stream = NodeStream::new(&node);
            let total = stream.fold(0.0, |acc, n| match n {
                Node::Number(Numeric::Float(price)) => acc + price,
                _ => acc,
            });
            println!("  Total: ${:.2}", total);
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
    println!();
}

/// Demonstrates path-based node access
fn demo_path_access() {
    println!("--- Example 5: Path-Based Access ---");

    let yaml = r#"
config:
  database:
    host: localhost
    port: 5432
    credentials:
      username: admin
      password: secret
  server:
    port: 8080
    threads: 4
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            // Access nested values using paths
            let mut path = NodePath::new();
            path.push("config");
            path.push("database");
            path.push("host");
            
            if let Some(found) = path.get(&node) {
                println!("Database host: {:?}", found);
            }

            // Access array elements
            let yaml2 = r#"
items:
  - name: First
    value: 100
  - name: Second
    value: 200
"#;
            let mut source2 = BufferSource::new(yaml2.as_bytes());
            if let Ok(node2) = parse(&mut source2) {
                let mut path2 = NodePath::new();
                path2.push("items");
                path2.push(1usize);  // Second item
                path2.push("name");
                
                if let Some(found) = path2.get(&node2) {
                    println!("Second item name: {:?}", found);
                }
            }

            // Direct path segments
            let path3 = NodePath::from_segments(vec![
                PathSegment::from("config"),
                PathSegment::from("server"),
                PathSegment::from("port"),
            ]);
            
            if let Some(found) = path3.get(&node) {
                println!("Server port: {:?}", found);
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
    println!();
}

/// Demonstrates collecting specific node types
fn demo_collecting() {
    println!("--- Example 6: Collecting Specific Types ---");

    let yaml = r#"
user:
  name: Alice Johnson
  email: alice@example.com
  age: 30
  scores:
    - 95
    - 87
    - 92
  tags:
    - developer
    - admin
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            // Collect all strings
            println!("All strings in document:");
            let strings = node.collect_strings();
            for s in &strings {
                println!("  - {}", s);
            }
            println!("  Total: {} strings", strings.len());

            // Collect all integers
            println!("\nAll integers in document:");
            let numbers = node.collect_numbers();
            for n in &numbers {
                println!("  - {}", n);
            }
            println!("  Total: {} integers", numbers.len());

            // Count total nodes
            let total_nodes = node.count_nodes();
            println!("\nTotal nodes in tree: {}", total_nodes);
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
    println!();
}

/// Demonstrates processing large documents efficiently
fn demo_large_document() {
    println!("--- Example 7: Large Document Processing ---");

    // Generate a large YAML structure
    let mut yaml = String::from("items:\n");
    for i in 0..1000 {
        yaml.push_str(&format!("  - id: {}\n", i));
        yaml.push_str(&format!("    name: Item {}\n", i));
        yaml.push_str(&format!("    value: {}\n", i * 10));
    }

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("Parsed large document with 1000 items");

            // Count nodes efficiently using iterator
            let count = node.count_nodes();
            println!("  Total nodes: {}", count);

            // Find specific items using stream
            let stream = NodeStream::new(&node);
            let high_value_count = stream
                .filter(|n| {
                    matches!(n, Node::Number(Numeric::Integer(v)) if *v > 5000)
                })
                .count();
            println!("  Items with value > 5000: {}", high_value_count);

            // Calculate statistics using fold
            let stream = NodeStream::new(&node);
            let (sum, count) = stream.fold((0i64, 0usize), |(sum, count), n| {
                match n {
                    Node::Number(Numeric::Integer(v)) => (sum + v, count + 1),
                    _ => (sum, count),
                }
            });
            
            if count > 0 {
                let average = sum as f64 / count as f64;
                println!("  Average value: {:.2}", average);
            }

            println!("\nStreaming allows processing large documents efficiently");
            println!("without loading everything into memory at once.");
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
    println!();
}
