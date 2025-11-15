//! String Interning Example
//!
//! This example demonstrates how to use string interning to reduce memory usage
//! when working with YAML documents that contain many repeated strings.

use yaml_lib::{StringInterner, CommonStrings};

fn main() {
    println!("=== String Interning Demo ===\n");

    demo_basic_interning();
    println!();
    demo_memory_savings();
    println!();
    demo_common_strings();
    println!();
    demo_performance_stats();
}

/// Demonstrate basic string interning
fn demo_basic_interning() {
    println!("1. Basic String Interning");
    println!("--------------------------");

    let interner = StringInterner::new();

    // Intern some strings
    let s1 = interner.intern("name");
    let s2 = interner.intern("name");
    let _s3 = interner.intern("type");

    println!("Interned 'name' twice and 'type' once");
    println!("s1 == s2: {}", s1.as_str() == s2.as_str());
    println!("s1 ref count: {}", s1.ref_count());
    println!("s2 ref count: {}", s2.ref_count());
    println!("Unique strings: {}", interner.len());
}

/// Demonstrate memory savings with repeated strings
fn demo_memory_savings() {
    println!("2. Memory Savings");
    println!("-----------------");

    let interner = StringInterner::new();
    let mut references = Vec::new();

    // Simulate a YAML document with repeated keys
    let keys = ["name", "type", "value", "id", "status", "config"];
    
    println!("Simulating 100 objects with 6 keys each...");
    for _ in 0..100 {
        for &key in &keys {
            references.push(interner.intern(key));
        }
    }

    let (total_bytes, interned_bytes, savings, percent) = interner.memory_savings();

    println!("Total bytes (without interning): {} bytes", total_bytes);
    println!("Interned bytes (with interning):  {} bytes", interned_bytes);
    println!("Savings:                          {} bytes ({:.1}%)", savings, percent);
    println!("Unique strings:                   {}", interner.len());
    println!("Total references:                 {}", references.len());
}

/// Demonstrate pre-interned common strings
fn demo_common_strings() {
    println!("3. Common Strings");
    println!("-----------------");

    let common = CommonStrings::new();

    println!("Pre-interned common YAML keys:");
    println!("  - name:    '{}'", common.name.as_str());
    println!("  - type:    '{}'", common.type_.as_str());
    println!("  - id:      '{}'", common.id.as_str());
    println!("  - value:   '{}'", common.value.as_str());
    println!("  - version: '{}'", common.version.as_str());
    println!("  - config:  '{}'", common.config.as_str());
    println!("  - data:    '{}'", common.data.as_str());
    println!("  - status:  '{}'", common.status.as_str());

    println!("\nThese can be used directly without interning overhead");
}

/// Demonstrate performance statistics
fn demo_performance_stats() {
    println!("4. Performance Statistics");
    println!("-------------------------");

    let interner = StringInterner::new();
    let mut _refs = Vec::new();

    // Intern strings with varying hit rates
    for i in 0..1000 {
        let key = match i % 10 {
            0..=5 => "common_key",      // 60% hit rate
            6..=8 => "medium_key",       // 30% hit rate  
            _ => &format!("unique_{}", i), // 10% unique
        };
        _refs.push(interner.intern(key));
    }

    let stats = interner.stats();
    println!("Total lookups:    {}", stats.hits + stats.misses);
    println!("Cache hits:       {} ({:.1}%)", stats.hits, stats.hit_rate());
    println!("Cache misses:     {}", stats.misses);
    println!("Unique strings:   {}", stats.unique_strings);
}
