//! Example demonstrating embedded systems support
//!
//! Shows how to use the YAML library in resource-constrained environments
//! with no_std, custom allocators, and strict resource limits.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "std")]
use yaml_lib::{parse, BufferSource, Node};

#[cfg(feature = "std")]
use yaml_lib::embedded::{
    allocator::BumpAllocator,
    config::*,
    lightweight_node::{LightNode, NodeArena},
    limits::NodeValidator,
};

#[cfg(feature = "std")]
fn main() {
    println!("=== YAML Embedded Systems Examples ===\n");

    demo_resource_limits();
    demo_custom_allocators();
    demo_lightweight_nodes();
    demo_validation();
    demo_bounded_parsing();
}

#[cfg(not(feature = "std"))]
fn main() {
    // In a real embedded environment, you would use your platform's output
    // mechanism instead of println!
}

#[cfg(feature = "std")]
fn demo_resource_limits() {
    println!("--- Example 1: Resource Limits ---");

    println!("Embedded configuration constants:");
    println!("  MAX_NESTING_DEPTH: {}", MAX_NESTING_DEPTH);
    println!("  MAX_DOCUMENT_SIZE: {} bytes", MAX_DOCUMENT_SIZE);
    println!("  MAX_STRING_LENGTH: {} bytes", MAX_STRING_LENGTH);
    println!("  MAX_SEQUENCE_ITEMS: {}", MAX_SEQUENCE_ITEMS);
    println!("  MAX_MAPPING_PAIRS: {}", MAX_MAPPING_PAIRS);
    println!("  MAX_ANCHORS: {}", MAX_ANCHORS);
    println!();
}

#[cfg(feature = "std")]
fn demo_custom_allocators() {
    println!("--- Example 2: Custom Allocators ---");

    // Bump allocator demonstration
    println!("Bump Allocator:");
    let _allocator = BumpAllocator::new();
    println!("  Bump allocator is available for no_std environments");
    println!("  Provides O(1) allocation with minimal overhead");
    println!("  Can be reset to reuse memory for temporary allocations");

    // Fixed-size pool demonstration
    println!("\nFixed-Size Pool:");
    println!("  Fixed-size pool allocator is available");
    println!("  Efficient for allocating many same-sized objects");
    println!("  Reduces fragmentation in long-running embedded systems");
    println!("  Example: FixedSizePool::<[u8; 32], 64>::new()");
    println!();
}

#[cfg(feature = "std")]
fn demo_lightweight_nodes() {
    println!("--- Example 3: Lightweight Nodes ---");

    // LightNode is a memory-efficient alternative to full Node
    let node1 = LightNode::string("sensor_data").unwrap();
    let node2 = LightNode::integer(42);
    let node3 = LightNode::float(3.14);

    println!("LightNode sizes (stack allocated):");
    println!("  String node: {} bytes", core::mem::size_of_val(&node1));
    println!("  Integer node: {} bytes", core::mem::size_of_val(&node2));
    println!("  Float node: {} bytes", core::mem::size_of_val(&node3));

    let full_node_size = core::mem::size_of::<Node>();
    println!("\nFor comparison:");
    println!("  Full Node size: {} bytes", full_node_size);
    println!("  LightNode is much more compact for embedded systems!");

    // Arena allocation demonstration
    println!("\nNode Arena:");
    let mut arena = NodeArena::new();
    println!("  Arena created for managing collections of lightweight nodes");
    println!("  Supports efficient batch allocation of arrays and mappings");

    // Create a small array in the arena
    let array_data = alloc::vec![
        LightNode::integer(1),
        LightNode::integer(2),
        LightNode::integer(3),
    ];
    match arena.add_array(array_data) {
        Ok(id) => println!("  Created array with ID: {}", id),
        Err(e) => println!("  Error: {}", e),
    }
    println!();
}

#[cfg(feature = "std")]
fn demo_validation() {
    println!("--- Example 4: Node Validation ---");

    // Create a simple document
    let yaml = r#"
sensor_config:
  temperature:
    enabled: true
    threshold: 25
  humidity:
    enabled: true
    threshold: 70
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    let doc = parse(&mut source).unwrap();

    // Validate against embedded constraints
    let mut validator = NodeValidator::new();

    match validator.validate(&doc) {
        Ok(()) => println!("✓ Document passed all validation checks"),
        Err(e) => println!("✗ Validation failed: {}", e),
    }

    // Try with deeply nested structure (should pass)
    let nested = create_nested_structure(30);
    match validator.validate(&nested) {
        Ok(()) => println!(
            "✓ 30-level nesting is within limits (max: {})",
            MAX_NESTING_DEPTH
        ),
        Err(e) => println!("✗ Validation failed: {}", e),
    }

    // Try with too deep nesting (should fail if it exceeds limits)
    let too_deep = create_nested_structure(MAX_NESTING_DEPTH + 10);
    match validator.validate(&too_deep) {
        Ok(()) => println!("✓ Deep nesting passed"),
        Err(e) => println!("✗ Expected failure for excessive nesting: {}", e),
    }
    println!();
}

#[cfg(feature = "std")]
fn demo_bounded_parsing() {
    println!("--- Example 5: Bounded Resource Parsing ---");

    // Example: IoT sensor configuration
    let config_yaml = r#"
device:
  id: "sensor-001"
  type: "temperature"
  location: "room-a"
  settings:
    interval: 60
    unit: "celsius"
    precision: 2
  thresholds:
    min: 15
    max: 30
    alert: true
"#;

    println!("Parsing IoT device configuration...");
    println!("YAML size: {} bytes", config_yaml.len());

    // Check size before parsing
    if config_yaml.len() > MAX_DOCUMENT_SIZE {
        println!(
            "✗ Configuration exceeds MAX_DOCUMENT_SIZE ({} bytes)",
            MAX_DOCUMENT_SIZE
        );
        return;
    }

    let mut source = BufferSource::new(config_yaml.as_bytes());
    match parse(&mut source) {
        Ok(doc) => {
            println!("✓ Configuration parsed successfully");

            // Validate
            let mut validator = NodeValidator::new();
            match validator.validate(&doc) {
                Ok(()) => {
                    println!("✓ Configuration validated");

                    // Show structure
                    println!("\nConfiguration structure:");
                    doc.visit(|node, depth| {
                        let indent = "  ".repeat(depth);
                        match node {
                            Node::Str(s, _, _) => {
                                if !s.starts_with("---") && !s.is_empty() {
                                    println!("{}- \"{}\"", indent, s);
                                }
                            }
                            Node::Number(n) => println!("{}- {:?}", indent, n),
                            Node::Boolean(b) => println!("{}- {}", indent, b),
                            _ => {}
                        }
                        depth < MAX_NESTING_DEPTH // Safety check
                    });
                }
                Err(e) => println!("✗ Validation failed: {}", e),
            }
        }
        Err(e) => println!("✗ Parse error: {}", e),
    }
    println!();
}

// Helper function to create nested structures for testing
#[cfg(feature = "std")]
fn create_nested_structure(depth: usize) -> Node {
    fn build_nested(current_depth: usize, max_depth: usize) -> Node {
        if current_depth >= max_depth {
            Node::from(42)
        } else {
            Node::Mapping(alloc::vec![
                (Node::from("level"), Node::from(current_depth as i32)),
                (
                    Node::from("nested"),
                    build_nested(current_depth + 1, max_depth)
                ),
            ])
        }
    }

    Node::Documents(alloc::vec![Node::Document(alloc::vec![build_nested(
        0, depth
    )])])
}

// Additional examples for real embedded usage

/// Example: Parsing sensor readings in constrained memory
#[cfg(feature = "std")]
fn parse_sensor_reading() {
    let sensor_data = r#"
reading:
  temperature: 23.5
  humidity: 65
  timestamp: 1699900000
"#;

    if sensor_data.len() <= MAX_DOCUMENT_SIZE {
        let mut source = BufferSource::new(sensor_data.as_bytes());
        if let Ok(doc) = parse(&mut source) {
            // Process the reading...
            let mut validator = NodeValidator::new();
            if validator.validate(&doc).is_ok() {
                // Safe to use
                println!("Sensor reading processed");
            }
        }
    }
}

/// Example: Configuration with bounded arrays
#[cfg(feature = "std")]
fn validate_configuration() {
    let config = r#"
endpoints:
  - "server1.local"
  - "server2.local"
  - "server3.local"
"#;

    let mut source = BufferSource::new(config.as_bytes());
    if let Ok(doc) = parse(&mut source) {
        let mut validator = NodeValidator::new();
        match validator.validate(&doc) {
            Ok(()) => println!("✓ Configuration is valid"),
            Err(e) => println!("✗ Invalid configuration: {}", e),
        }
    }
}
