//! Performance Optimization Examples
//!
//! This example demonstrates various performance optimization techniques
//! available in the YAML library.

use yaml_lib::*;

fn main() {
    println!("=== YAML Performance Optimization Examples ===\n");

    demo_lazy_tags();
    println!();
    demo_capacity_hints();
    println!();
    demo_string_pool();
    println!();
    demo_fast_path_detection();
    println!();
    demo_node_builder();
    println!();
    demo_performance_optimizer();
}

/// Demonstrate lazy tag coercion
fn demo_lazy_tags() {
    println!("1. Lazy Tag Coercion");
    println!("--------------------");
    println!("Defer type conversion until the value is actually needed");

    let mut lazy_int = LazyTag::new("42".to_string(), "!!int".to_string());
    println!("Created lazy tag for '42' with !!int");
    println!("  Is coerced? {}", lazy_int.is_coerced());

    // Value is only coerced when accessed
    {
        let _value = lazy_int.get_or_coerce();
        println!("  Accessed value (coercion happens now)");
    }
    println!("  Is coerced? {}", lazy_int.is_coerced());

    let mut lazy_bool = LazyTag::new("true".to_string(), "!!bool".to_string());
    {
        let _value = lazy_bool.get_or_coerce();
        println!("  Lazy bool coerced to: {:?}", _value);
    }
}

/// Demonstrate capacity hints for pre-allocation
fn demo_capacity_hints() {
    println!("2. Capacity Hints");
    println!("-----------------");
    println!("Pre-allocate collections to avoid repeated reallocations");

    // Default hints
    let hints = CapacityHints::new();
    println!("Default hints:");
    println!("  Mapping pairs:   {}", hints.mapping_pairs);
    println!("  Sequence items:  {}", hints.sequence_items);
    println!("  String capacity: {}", hints.string_capacity);

    // Small document hints
    let small = CapacityHints::small();
    println!("\nSmall document hints:");
    println!("  Mapping pairs:   {}", small.mapping_pairs);
    println!("  Sequence items:  {}", small.sequence_items);

    // Large document hints
    let large = CapacityHints::large();
    println!("\nLarge document hints:");
    println!("  Mapping pairs:   {}", large.mapping_pairs);
    println!("  Sequence items:  {}", large.sequence_items);

    // Adaptive hints
    let mut adaptive = CapacityHints::new();
    println!("\nAdaptive hints (learns from actual usage):");
    println!(
        "  Before: mapping={}, sequence={}",
        adaptive.mapping_pairs, adaptive.sequence_items
    );
    adaptive.update(20, 15);
    println!(
        "  After update(20, 15): mapping={}, sequence={}",
        adaptive.mapping_pairs, adaptive.sequence_items
    );
}

/// Demonstrate string pooling
fn demo_string_pool() {
    println!("3. String Interning");
    println!("-------------------");
    println!("Deduplicate common strings during parsing");

    let interner = StringInterner::new();

    // Intern the same strings
    let s1 = interner.intern("name");
    let _s2 = interner.intern("type");
    let s3 = interner.intern("name"); // Reuses existing

    println!("Added 'name', 'type', 'name' to interner");
    println!("  Unique strings: {}", interner.len());
    println!(
        "  'name' references share same memory: {}",
        s1.as_str().as_ptr() == s3.as_str().as_ptr()
    );

    // Simulate parsing 100 objects with repeated keys
    for _ in 0..100 {
        let _name = interner.intern("name");
        let _type = interner.intern("type");
        let _value = interner.intern("value");
        let _id = interner.intern("id");
    }

    println!("\nAfter processing 100 objects:");
    println!(
        "  Unique strings: {} (only 4 allocations for 400 strings!)",
        interner.len()
    );
}

/// Demonstrate fast path detection
fn demo_fast_path_detection() {
    println!("4. Fast Path Detection");
    println!("----------------------");
    println!("Detect simple patterns that can use optimized code paths");

    // Test simple scalars
    let test_cases = [
        ("hello", "simple scalar"),
        ("hello_world", "simple scalar with underscore"),
        ("hello world", "not simple (has space)"),
        ("123", "simple integer"),
        ("-456", "negative integer"),
        ("12.34", "not simple integer (has decimal)"),
    ];

    for (input, description) in test_cases.iter() {
        let is_simple_scalar = FastPathDetector::is_simple_scalar(input);
        let is_simple_int = FastPathDetector::is_simple_int(input);
        println!("  '{}' ({})", input, description);
        println!(
            "    Simple scalar: {}, Simple int: {}",
            is_simple_scalar, is_simple_int
        );
    }

    // Test document-level fast path
    let simple_doc = "name: John\nage: 30\ncity: NYC";
    let complex_doc = "name: &anchor John\nage: 30\nalias: *anchor";

    println!("\nDocument fast path detection:");
    println!(
        "  Simple doc can use fast path: {}",
        FastPathDetector::can_use_fast_path(simple_doc)
    );
    println!(
        "  Complex doc can use fast path: {}",
        FastPathDetector::can_use_fast_path(complex_doc)
    );
}

/// Demonstrate node builder with capacity management
fn demo_node_builder() {
    println!("5. Node Builder");
    println!("---------------");
    println!("Memory-efficient node construction with reusable buffers");

    let mut builder = NodeBuilder::new();

    println!("Builder capacity hints:");
    println!("  Mapping pairs:  {}", builder.hints().mapping_pairs);
    println!("  Sequence items: {}", builder.hints().sequence_items);

    // Build nodes with pre-allocated capacity
    let _array = builder.build_array_with_capacity(10);
    let _mapping = builder.build_mapping_with_capacity(5);

    println!("\nBuilt array and mapping with pre-allocated capacity");
    println!("  This avoids reallocations during construction");

    // Update hints based on observed usage
    builder.update_hints(15, 20);
    println!("\nUpdated hints based on usage:");
    println!("  Mapping pairs:  {}", builder.hints().mapping_pairs);
    println!("  Sequence items: {}", builder.hints().sequence_items);
}

/// Demonstrate the complete performance optimizer
fn demo_performance_optimizer() {
    println!("6. Performance Optimizer");
    println!("------------------------");
    println!("Combines multiple optimization strategies");

    // Default optimizer
    let default_opt = PerformanceOptimizer::new();
    println!("Default optimizer:");
    println!("  Lazy tags:  {}", default_opt.lazy_tags);
    println!("  Zero copy:  {}", default_opt.zero_copy);
    println!("  String interner: {}", default_opt.string_interner.is_some());

    // Aggressive optimizer
    let aggressive = PerformanceOptimizer::aggressive();
    println!("\nAggressive optimizer:");
    println!("  Lazy tags:  {}", aggressive.lazy_tags);
    println!("  Zero copy:  {}", aggressive.zero_copy);
    println!("  String interner: {}", aggressive.string_interner.is_some());
    println!("  Mapping capacity: {}", aggressive.hints.mapping_pairs);

    // Custom optimizer
    let mut custom = PerformanceOptimizer::new();
    custom.enable_lazy_tags();
    custom.enable_zero_copy();
    custom.enable_string_interning(128);

    println!("\nCustom optimizer:");
    println!("  Lazy tags:  {}", custom.lazy_tags);
    println!("  Zero copy:  {}", custom.zero_copy);
    println!("  String interner: {}", custom.string_interner.is_some());

    // Use optimizer to allocate collections
    let _vec = custom.alloc_vec::<Node>();
    let _string = custom.alloc_string();
    println!("\nAllocated collections with optimized capacity");
}
