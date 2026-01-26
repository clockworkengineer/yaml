
//! Example demonstrating performance measurement and optimization utilities
//!
//! Shows how to use DocumentStats, Timer, and Profiler for analyzing
//! and optimizing YAML processing performance.

use yaml_lib::{
    parse, stringify, BufferDestination, BufferSource, DocumentStats, Node, Profiler, Timer,
};

fn main() {
    println!("=== YAML Performance Measurement Examples ===\n");

    demo_document_stats();
    demo_simple_timing();
    demo_profiler();
    demo_memory_analysis();
    demo_optimization_comparison();
}

/// Demonstrates document statistics gathering
fn demo_document_stats() {
    println!("--- Example 1: Document Statistics ---");

    let yaml = r#"
users:
  - name: Alice
    age: 30
    roles: [admin, user]
  - name: Bob
    age: 25
    roles: [user]
config:
  database:
    host: localhost
    port: 5432
    settings:
      pool_size: 10
      timeout: 30
  cache:
    enabled: true
    ttl: 3600
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    // Gather statistics
    let stats = DocumentStats::from_node(&doc);

    println!("Document Analysis:");
    println!("  Total nodes: {}", stats.total_nodes);
    println!("  Max depth: {}", stats.max_depth);
    println!(
        "  Strings: {} ({} bytes)",
        stats.string_count, stats.total_string_bytes
    );
    println!("  Numbers: {}", stats.number_count);
    println!("  Booleans: {}", stats.boolean_count);
    println!(
        "  Arrays: {} (largest: {})",
        stats.array_count, stats.largest_array
    );
    println!(
        "  Mappings: {} (largest: {})",
        stats.mapping_count, stats.largest_mapping
    );
    println!(
        "  Estimated memory: {} bytes",
        stats.estimated_memory_bytes()
    );
    println!("\n{}\n", stats.summary());
}

/// Demonstrates simple timing of operations
fn demo_simple_timing() {
    println!("--- Example 2: Simple Operation Timing ---");

    let yaml = generate_large_yaml(100);

    // Time parsing
    let timer = Timer::new("Parse 100-item YAML");
    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();
    let parse_time = timer.stop();
    println!("Parsing took: {:?}", parse_time);

    // Time stringification
    let timer = Timer::new("Stringify document");
    let mut dest = BufferDestination::new();
    let _ = stringify(&doc, &mut dest);
    let stringify_time = timer.stop();
    println!("Stringifying took: {:?}", stringify_time);

    // Time statistics gathering
    let timer = Timer::new("Gather statistics");
    let _stats = DocumentStats::from_node(&doc);
    let stats_time = timer.stop();
    println!("Stats gathering took: {:?}\n", stats_time);
}

/// Demonstrates profiler for multi-operation tracking
fn demo_profiler() {
    println!("--- Example 3: Multi-Operation Profiler ---");

    let mut profiler = Profiler::new();

    // Profile a complete workflow
    let yaml = profiler.time("Generate YAML", || generate_large_yaml(50));

    let doc = profiler.time("Parse YAML", || {
        let mut source = BufferSource::new(yaml.as_bytes());
        parse(&mut source).unwrap()
    });

    let _stats = profiler.time("Analyze document", || DocumentStats::from_node(&doc));

    let _yaml_out = profiler.time("Stringify document", || {
        let mut dest = BufferDestination::new();
        let _ = stringify(&doc, &mut dest);
        dest
    });

    profiler.time("Tree traversal", || {
        doc.visit(|_node, _depth| true);
    });

    // Print all measurements
    profiler.print_results();
    println!();
}

/// Demonstrates memory usage analysis for different document structures
fn demo_memory_analysis() {
    println!("--- Example 4: Memory Usage Analysis ---");

    // Compare memory usage of different structures
    let structures = vec![
        ("Flat mapping", create_flat_mapping(100)),
        ("Nested mappings", create_nested_structure(10)),
        ("Large arrays", create_array_heavy(50)),
        ("String heavy", create_string_heavy(30)),
    ];

    println!(
        "{:<20} {:>15} {:>15} {:>10}",
        "Structure", "Total Nodes", "Est. Memory", "Max Depth"
    );
    println!("{:-<62}", "");

    for (name, doc) in structures {
        let stats = DocumentStats::from_node(&doc);
        println!(
            "{:<20} {:>15} {:>12} KB {:>10}",
            name,
            stats.total_nodes,
            stats.estimated_memory_bytes() / 1024,
            stats.max_depth
        );
    }
    println!();
}

/// Demonstrates comparing different optimization approaches
fn demo_optimization_comparison() {
    println!("--- Example 5: Optimization Comparison ---");

    let yaml = generate_large_yaml(200);
    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    println!("Comparing different node counting approaches:\n");

    // Approach 1: Using built-in count_nodes()
    let timer = Timer::new("Built-in count_nodes()");
    let count1 = doc.count_nodes();
    let time1 = timer.stop();
    println!("  Method 1 (built-in): {} nodes in {:?}", count1, time1);

    // Approach 2: Manual counting with visit()
    let timer = Timer::new("Manual visit() counting");
    let mut count2 = 0;
    doc.visit(|_node, _depth| {
        count2 += 1;
        true
    });
    let time2 = timer.stop();
    println!("  Method 2 (manual):   {} nodes in {:?}", count2, time2);

    // Calculate speedup
    if time1 < time2 {
        let ratio = time2.as_secs_f64() / time1.as_secs_f64();
        println!("\n  Built-in method is {:.2}x faster!", ratio);
    } else {
        println!("\n  Methods have similar performance");
    }
}

// Helper functions to generate test data

fn generate_large_yaml(items: usize) -> String {
    let mut yaml = String::from("items:\n");
    for i in 0..items {
        yaml.push_str(&format!("  - id: {}\n", i));
        yaml.push_str(&format!("    name: Item {}\n", i));
        yaml.push_str(&format!("    value: {}\n", i * 10));
    }
    yaml
}

fn create_flat_mapping(size: usize) -> Node {
    let pairs: Vec<(Node, Node)> = (0..size)
        .map(|i| (Node::from(format!("key{}", i)), Node::from(i as i32)))
        .collect();
    Node::Documents(vec![Node::Document(vec![Node::Mapping(pairs)])])
}

fn create_nested_structure(depth: usize) -> Node {
    fn build_nested(current_depth: usize, max_depth: usize) -> Node {
        if current_depth >= max_depth {
            Node::from(42)
        } else {
            Node::Mapping(vec![
                (Node::from("level"), Node::from(current_depth as i32)),
                (
                    Node::from("nested"),
                    build_nested(current_depth + 1, max_depth),
                ),
            ])
        }
    }

    Node::Documents(vec![Node::Document(vec![build_nested(0, depth)])])
}

fn create_array_heavy(arrays: usize) -> Node {
    let items: Vec<Node> = (0..arrays)
        .map(|_| {
            Node::Array(vec![
                Node::from(1),
                Node::from(2),
                Node::from(3),
                Node::from(4),
                Node::from(5),
            ])
        })
        .collect();
    Node::Documents(vec![Node::Document(vec![Node::Array(items)])])
}

fn create_string_heavy(count: usize) -> Node {
    let pairs: Vec<(Node, Node)> = (0..count)
        .map(|i| {
            (
                Node::from(format!("This is a longer key string number {}", i)),
                Node::from(format!(
                    "And this is an even longer value string with more text {}",
                    i
                )),
            )
        })
        .collect();
    Node::Documents(vec![Node::Document(vec![Node::Mapping(pairs)])])
}
