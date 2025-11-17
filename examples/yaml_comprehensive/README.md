# YAML Library Examples Guide

This guide provides an overview of all examples available in the yaml-lib project, organized by feature category.

## Quick Start

The library includes numerous working examples demonstrating all features. This guide helps you find the right example for your needs.

## Examples by Category

### 1. Basic Parsing and Access

**Location:** `examples/yaml_parse_and_stringify`

Demonstrates:
- Basic parsing with `parse()` and `BufferSource`/`FileSource`
- Safe access patterns with `get()`, `get_key()`, `get_key_safe()`
- Type conversion with `as_str()`, `as_i64()`, `as_bool()`
- Stringify back to YAML format

**Run:**
```bash
cd examples/yaml_parse_and_stringify
cargo run
```

### 2. Fluent API / Builder Pattern

**Location:** `examples/yaml_fluent_api` (if exists)

Demonstrates:
- Building YAML with `Node::mapping().insert().build()`
- Creating arrays with `Node::array().push().build()`
- Constructing nested structures programmatically
- Using `Node::from()` for automatic type conversion

**Key API Pattern:**
```rust
let doc = Node::mapping()
    .insert("name", "Alice")
    .insert("age", 30)
    .insert("active", true)
    .build();
```

### 3. Validation

**Location:** `examples/yaml_validation`

Demonstrates:
- Schema validation with `SchemaValidator`
- Type checking with `TypeValidator`
- Range constraints with `RangeValidator`
- String validation with `LengthValidator` and `PatternValidator`
- Required fields with `RequiredValidator`

**Run:**
```bash
cd examples/yaml_validation
cargo run
```

### 4. Streaming

**Location:** `examples/yaml_streaming`

Demonstrates:
- Memory-efficient processing with `NodeStream`
- Progressive parsing of large documents
- Filtering and transforming on-the-fly
- Path-based navigation with `NodePath`

**Run:**
```bash
cd examples/yaml_streaming
cargo run
```

### 5. Error Handling

**Location:** `examples/yaml_error_handling`

Demonstrates:
- Error catching and reporting with `YamlError`
- Error codes and diagnostics with `ErrorCode`
- Line/column information for errors
- Recovery strategies with `ErrorCollection`

**Run:**
```bash
cd examples/yaml_error_handling
cargo run
```

### 6. Format Conversion

**Locations:**
- `examples/yaml_to_json` - YAML ↔ JSON
- `examples/yaml_to_xml` - YAML ↔ XML
- `examples/yaml_to_toml` - YAML ↔ TOML
- `examples/yaml_to_bencode` - YAML ↔ Bencode

Demonstrates converting between YAML and other data formats using the `format-converters` feature.

**Run:**
```bash
cd examples/yaml_to_json
cargo run
```

### 7. Advanced YAML Features

**Locations:**
- `examples/yaml_anchors_aliases` - Anchors and aliases (&, *)
- `examples/yaml_advanced_tags` - Custom type tags
- `examples/yaml_multi_document` - Multiple documents in one file

Demonstrates YAML 1.2 advanced features like anchors, aliases, tags, and multi-document streams.

### 8. Performance Optimization

**Locations:**
- `examples/yaml_performance` - Performance profiling
- `examples/yaml_string_interning` - Memory optimization
- `examples/yaml_performance_opts` - Optimization strategies

Demonstrates:
- String interning for memory savings
- Performance profiling with `Profiler`
- Capacity hints and fast paths
- Zero-copy string handling

### 9. Node Manipulation

**Location:** `examples/yaml_node_manipulation`

Demonstrates:
- Creating and modifying nodes
- Tree traversal patterns
- Safe mutation with `get_mut()`, `get_key_mut()`

**Run:**
```bash
cd examples/yaml_node_manipulation
cargo run
```

### 10. Tree Traversal

**Location:** `examples/yaml_tree_traversal`

Demonstrates:
- Depth-first and breadth-first traversal
- Recursive pattern matching
- Finding nodes by criteria

**Run:**
```bash
cd examples/yaml_tree_traversal
cargo run
```

### 11. Embedded Systems

**Locations:**
- `examples/yaml_embedded_systems` - Embedded optimizations
- `examples/yaml_embedded_safe` - no_std compatibility

Demonstrates:
- Using the library with `no_std`
- Memory limits for embedded
- `embedded` feature flag usage

### 12. Safe Access Patterns

**Location:** `examples/yaml_safe_access`

Demonstrates:
- Safe access methods that never panic
- Option/Result handling patterns
- Defensive programming with YAML

**Run:**
```bash
cd examples/yaml_safe_access
cargo run
```

## Common Patterns

### Parsing from String
```rust
use yaml_lib::{parse, BufferSource};

let yaml = "key: value";
let mut source = BufferSource::new(yaml.as_bytes());
let doc = parse(&mut source)?;
```

### Parsing from File
```rust
use yaml_lib::{parse, FileSource};

let mut source = FileSource::new("config.yaml")?;
let doc = parse(&mut source)?;
```

### Building with Builder
```rust
use yaml_lib::Node;

let doc = Node::mapping()
    .insert("name", "Alice")
    .insert("items", Node::array()
        .push(1)
        .push(2)
        .push(3)
        .build())
    .build();
```

### Safe Access
```rust
// Never panics - returns Option
if let Some(value) = doc.get_key("name") {
    if let Some(name) = value.as_str() {
        println!("Name: {}", name);
    }
}
```

### Stringify to Buffer
```rust
use yaml_lib::{stringify, BufferDestination};

let mut dest = BufferDestination::new();
stringify(&doc, &mut dest)?;
let yaml_string = dest.to_string();
```

## Running All Examples

To test all examples at once:

```bash
# From project root
cargo test --workspace --examples
```

## Documentation

For complete API documentation:

```bash
cargo doc --open --all-features
```

See also:
- **DEVELOPER_GUIDE.md** - Comprehensive developer documentation
- **COMPLIANCE.md** - YAML 1.2 specification compliance details
- **FEATURE_SUMMARY.md** - Feature flags and capabilities

## Learning Path

Recommended order:
1. Start with `yaml_parse_and_stringify` for basics
2. Try `yaml_safe_access` for safe patterns
3. Explore `yaml_fluent_api` for building documents
4. Check `yaml_validation` for schema validation
5. Look at `yaml_error_handling` for robust apps
6. Experiment with `yaml_streaming` for large files
7. Try format converters (`yaml_to_json`, etc.)
8. Dive into advanced features (anchors, tags, multi-doc)
9. Optimize with performance examples
10. Explore embedded systems examples if needed

## Troubleshooting

### Example Won't Compile
```bash
cd examples/<example_name>
cargo clean
cargo build --all-features
```

### Feature Not Available
Check that you're using the right feature flags:
```bash
cargo run --all-features
```

### Can't Find Example
List all available examples:
```bash
ls examples/
```

## Contributing Examples

To add a new example:
1. Create directory: `examples/yaml_<feature_name>`
2. Add `Cargo.toml` with dependency on `yaml_lib`
3. Create `src/main.rs` with clear demonstrations
4. Add example to workspace in root `Cargo.toml`
5. Document in this guide

## License

All examples are part of yaml-lib and share its license (MIT or Apache-2.0).
