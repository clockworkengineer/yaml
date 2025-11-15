# String Interning Example

This example demonstrates string interning for memory optimization when working with YAML documents.

## Overview

String interning is a technique that stores only one copy of each distinct string value. When parsing YAML documents with repeated keys (like "name", "type", "value"), this can significantly reduce memory usage.

## Running the Example

```bash
cargo run --example yaml_string_interning
```

## What It Demonstrates

### 1. Basic Interning
- How to create a `StringInterner`
- Interning strings returns the same reference for identical strings
- Checking reference counts

### 2. Memory Savings
- Simulates parsing 100 YAML objects with repeated keys
- Shows memory usage comparison: with vs without interning
- Can save 20-40% memory for typical YAML documents

### 3. Common Strings
- Pre-interned common YAML keys ("name", "type", "id", etc.)
- Available through `CommonStrings` for zero-cost reuse
- No need to intern these strings yourself

### 4. Performance Statistics
- Cache hit rate tracking
- Unique string count
- Useful for tuning and optimization

## Key Types

- `StringInterner`: Thread-safe interner using `Arc` and `RwLock`
- `InternedString`: Reference-counted string wrapper
- `CommonStrings`: Pre-interned common keys
- `InternerStats`: Performance metrics

## Use Cases

1. **Configuration Parsing**: Config files often have repeated keys across many sections
2. **Data Serialization**: JSON/YAML APIs with consistent field names
3. **AST/IR**: Compiler/interpreter symbol tables
4. **Large Documents**: Multi-document YAML streams with shared vocabulary

## Memory Trade-offs

**Benefits:**
- Reduces memory for repeated strings (20-40% typical savings)
- Faster string equality checks (pointer comparison)
- Shared strings across threads (with `Arc`)

**Costs:**
- Small overhead per interned string (Arc pointer + refcount)
- Best for strings used 3+ times
- Thread synchronization cost (RwLock) for concurrent access

## Integration with YAML Parsing

While this example shows standalone usage, you can integrate string interning with YAML parsing:

```rust
use yaml_lib::{parse, StringInterner, BufferSource};

let interner = StringInterner::new();
let yaml = "name: John\ntype: user\nname: Jane\ntype: admin";
let source = BufferSource::new(yaml.as_bytes());

// Parse and intern keys
let doc = parse(source)?;
// ... manually intern string keys when processing the tree
```

For automatic interning during parsing, this would require modifying the parser to accept an optional `StringInterner` parameter.
