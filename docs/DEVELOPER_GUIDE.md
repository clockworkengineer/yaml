# YAML Library Developer Guide

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Core Concepts](#core-concepts)
5. [Feature Guides](#feature-guides)
6. [API Reference](#api-reference)
7. [Best Practices](#best-practices)
8. [Performance Tips](#performance-tips)
9. [Troubleshooting](#troubleshooting)

## Introduction

This YAML library provides a complete, production-ready YAML 1.2 implementation for Rust with:

- ✅ Full YAML 1.2 specification compliance
- ✅ No_std support with optional `alloc`
- ✅ Fluent API for building documents
- ✅ Advanced validation and schema support
- ✅ Streaming for large documents
- ✅ Comprehensive error handling with recovery
- ✅ Developer tools (debugging, inspection, diffing)
- ✅ Multiple format converters (JSON, XML, TOML, Bencode)

### Minimum Supported Rust Version

MSRV: 1.88.0

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
yaml_lib = "0.2.1"
```

### Feature Flags

```toml
[dependencies.yaml_lib]
version = "0.2.0"
features = [
    "std",              # Standard library (default)
    "alloc",            # Allocation support (default)
    "embedded",         # Embedded systems optimizations
    "parse-only",       # Only parsing, no serialization
    "stringify",        # YAML output (requires alloc)
    "format-converters", # JSON, XML, TOML, Bencode
    "file-io",          # File I/O operations
]
```

## Quick Start

### Parsing YAML

```rust
use yaml_lib::{parse_string, Node};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = r#"
        name: Alice
        age: 30
        hobbies:
          - reading
          - coding
    "#;
    
    let doc = parse_string(yaml)?;
    
    // Access values safely
    if let Some(name) = doc.get_key_safe("name") {
        println!("Name: {}", name.as_str().unwrap_or(""));
    }
    
    Ok(())
}
```

### Building YAML with Fluent API

```rust
use yaml_lib::Node;

fn main() {
    let doc = Node::mapping()
        .insert("name", "Alice")
        .insert("age", 30)
        .insert("hobbies", Node::array()
            .push("reading")
            .push("coding")
        )
        .build();
    
    println!("{:?}", doc);
}
```

### Stringifying to YAML

```rust
use yaml_lib::{stringify_to_string, Node};

fn main() -> Result<(), String> {
    let doc = Node::mapping()
        .insert("greeting", "Hello, World!")
        .build();
    
    let yaml = stringify_to_string(&doc)?;
    println!("{}", yaml);
    
    Ok(())
}
```

## Core Concepts

### Centralized Error Handling (Contributor Note)

All parser and lexer error messages must use the centralized helpers in `parser/document/error_builder.rs` (e.g., `syntax_error`, `structure_error`, `limit_error`, `forbidden_error`).

**Do not return raw error strings.**

This ensures all errors are consistent, include context, and are easy to maintain. See the module-level docs in `error_builder.rs` for usage examples and extension guidelines.


### Node Type

The `Node` enum represents all YAML values:

```rust
pub enum Node {
    None,                           // null
    Boolean(bool),                  // true/false
    Number(Numeric),                // integers, floats
    Str(String, StrKind, QuoteType), // strings
    Array(Vec<Node>),               // sequences
    Mapping(Vec<(Node, Node)>),     // mappings
    Set(Vec<Node>),                 // sets
    Tagged(Box<Node>, String),      // tagged values
    Anchored(Box<Node>, String),    // anchored values
    Alias(String),                  // aliases
    Comment(String),                // comments
    Document(Vec<Node>),            // single document
    Documents(Vec<Node>),           // multi-document
}
```

### Safe Access

Always use safe access methods to prevent panics:

```rust
// Safe key access
if let Some(value) = node.get_key_safe("key") {
    println!("Found: {:?}", value);
}

// Safe array access
if let Some(item) = node.get_safe(0) {
    println!("First item: {:?}", item);
}

// Safe type conversion
if let Some(s) = node.as_str() {
    println!("String value: {}", s);
}
```

## Feature Guides

### 1. Fluent API

Build complex documents naturally:

```rust
let config = Node::mapping()
    .insert("server", Node::mapping()
        .insert("host", "localhost")
        .insert("port", 8080)
        .insert("ssl", true)
    )
    .insert("database", Node::mapping()
        .insert("driver", "postgres")
        .insert("connection", Node::mapping()
            .insert("host", "db.example.com")
            .insert("port", 5432)
        )
    )
    .insert("features", Node::array()
        .push("authentication")
        .push("logging")
        .push("metrics")
    )
    .build();
```

### 2. Validation and Schema

Define and validate document structure:

```rust
use yaml_lib::{Schema, SchemaType, SchemaValidator, ValidationContext};

// Define schema
let schema = Schema::object()
    .required("name", Schema::string())
    .required("age", Schema::integer().min(0).max(150))
    .optional("email", Schema::string().pattern(r"^[\w\.-]+@[\w\.-]+\.\w+$"))
    .build();

// Validate document
let validator = SchemaValidator::new(schema);
let mut context = ValidationContext::new();

match validator.validate(&doc, &mut context) {
    Ok(_) => println!("Valid!"),
    Err(e) => println!("Validation failed: {}", e),
}
```

### 3. Streaming Large Documents

Process large documents efficiently:

```rust
use yaml_lib::{NodeStream, Node};

let large_doc = /* ... */;

// Filter and process without loading all into memory
let numbers: Vec<i64> = NodeStream::new(&large_doc)
    .filter(|n| matches!(n, Node::Number(_)))
    .map(|n| match n {
        Node::Number(Numeric::Integer(i)) => *i,
        _ => 0,
    })
    .collect();
```

### 4. Error Handling with Recovery

Handle errors gracefully:

```rust
use yaml_lib::{parse_string_with_recovery, RecoveryHandler, RecoveryStrategy};

let yaml = "malformed: yaml: document:";

let handler = RecoveryHandler::lenient()
    .with_strategy(RecoveryStrategy::UseDefault);

match parse_string_with_recovery(yaml, handler) {
    Ok((doc, errors)) => {
        println!("Parsed with {} errors", errors.len());
        for err in errors {
            println!("  - {}", err);
        }
    }
    Err(e) => println!("Fatal error: {}", e),
}
```

### 5. Developer Tools

Debug and inspect documents:

```rust
use yaml_lib::{print_tree, diff_nodes, NodeInfo, Tracer};

// Print document structure
println!("{}", print_tree(&doc));

// Get node information
let info = NodeInfo::new(&doc);
println!("{}", info.format());

// Compare documents
let result = diff_nodes(&old_doc, &new_doc);
println!("{}", result.format());

// Trace execution
let mut tracer = Tracer::new();
tracer.enter("parse".to_string(), "Starting parse".to_string());
// ... operations ...
tracer.exit("parse".to_string(), "Done".to_string());
println!("{}", tracer.format());
```

### 6. Custom Serialization

Control output formatting:

```rust
use yaml_lib::{FormatOptions, StreamingSerializer, stringify_with_options};

// Custom format
let options = FormatOptions::pretty()
    .with_indent(4)
    .with_sorted_keys(true)
    .with_explicit_markers(true);

let yaml = stringify_with_options(&doc, options)?;

// Streaming serialization
let mut serializer = StreamingSerializer::new(&mut output)
    .with_options(options);

serializer.serialize_node(&doc)?;
```

### 7. Format Conversion

Convert between formats:

```rust
use yaml_lib::{to_json, to_xml, to_toml};

// YAML to JSON
let json = to_json(&doc)?;

// YAML to XML
let xml = to_xml(&doc)?;

// YAML to TOML
let toml = to_toml(&doc)?;
```

## API Reference

### Parsing Functions

```rust
// Parse from string
pub fn parse_string(yaml: &str) -> Result<Node, YamlError>

// Parse from file
pub fn parse_file(path: &str) -> Result<Node, YamlError>

// Parse with custom config
pub fn parse_with_config(
    yaml: &str,
    config: ParserConfig
) -> Result<Node, YamlError>

// Parse with error recovery
pub fn parse_string_with_recovery(
    yaml: &str,
    handler: RecoveryHandler
) -> Result<(Node, Vec<YamlError>), YamlError>
```

### Node Builders

```rust
impl Node {
    // Create builders
    pub fn mapping() -> MappingBuilder
    pub fn array() -> ArrayBuilder
    pub fn set() -> SetBuilder
    
    // Direct construction
    pub fn from<T: Into<Node>>(value: T) -> Node
    
    // Safe access
    pub fn get_safe(&self, index: usize) -> Option<&Node>
    pub fn get_key_safe(&self, key: &str) -> Option<&Node>
    pub fn get_mut_safe(&mut self, index: usize) -> Option<&mut Node>
    
    // Type checking
    pub fn is_null(&self) -> bool
    pub fn is_string(&self) -> bool
    pub fn is_number(&self) -> bool
    pub fn is_boolean(&self) -> bool
    pub fn is_sequence(&self) -> bool
    pub fn is_mapping(&self) -> bool
    
    // Type conversion
    pub fn as_str(&self) -> Option<&str>
    pub fn as_bool(&self) -> Option<bool>
    pub fn as_i64(&self) -> Option<i64>
    pub fn as_f64(&self) -> Option<f64>
}
```

### Serialization Functions

```rust
// Stringify to string
pub fn stringify_to_string(node: &Node) -> Result<String, String>

// Stringify to file
pub fn stringify_to_file(node: &Node, path: &str) -> Result<(), String>

// With custom options
pub fn stringify_with_options(
    node: &Node,
    options: FormatOptions
) -> Result<String, String>
```

## Best Practices

### 1. Always Use Safe Access

❌ **Don't:**
```rust
let value = &node["key"];  // Panics if key doesn't exist
```

✅ **Do:**
```rust
if let Some(value) = node.get_key_safe("key") {
    // Handle value
}
```

### 2. Handle Errors Explicitly

❌ **Don't:**
```rust
let doc = parse_string(yaml).unwrap();  // Panics on error
```

✅ **Do:**
```rust
match parse_string(yaml) {
    Ok(doc) => { /* use doc */ },
    Err(e) => eprintln!("Parse error: {}", e),
}
```

### 3. Use Builders for Complex Documents

❌ **Don't:**
```rust
let mut pairs = Vec::new();
pairs.push((Node::from("key1"), Node::from("value1")));
pairs.push((Node::from("key2"), Node::from("value2")));
let doc = Node::Mapping(pairs);
```

✅ **Do:**
```rust
let doc = Node::mapping()
    .insert("key1", "value1")
    .insert("key2", "value2")
    .build();
```

### 4. Validate Input Documents

✅ **Always validate untrusted input:**
```rust
let schema = Schema::object()
    .required("required_field", Schema::string())
    .build();

let validator = SchemaValidator::new(schema);
validator.validate(&doc, &mut ValidationContext::new())?;
```

### 5. Use Streaming for Large Documents

✅ **For documents > 10MB:**
```rust
let stream = NodeStream::new(&large_doc);
for node in stream.filter(|n| is_interesting(n)) {
    process(node);
}
```

## Performance Tips

### 1. String Interning

Enable string interning for documents with repeated strings:

```rust
use yaml_lib::StringInterner;

let mut interner = StringInterner::new();
// Parser will automatically use interner
```

**Benefit**: 30-50% memory reduction for documents with many repeated strings.

### 2. Parser Configuration

Optimize parser for your use case:

```rust
use yaml_lib::ParserConfig;

// For embedded systems
let config = ParserConfig::embedded();

// For permissive parsing
let config = ParserConfig::permissive();

// Custom limits
let config = ParserConfig::new()
    .with_max_depth(50)
    .with_max_size(100000);
```

### 3. Streaming Serialization

For large documents, use streaming:

```rust
let mut serializer = StreamingSerializer::new(&mut output)
    .with_buffer_size(8192);  // Larger buffer = fewer flushes

serializer.serialize_node(&large_doc)?;
```

### 4. Reuse Allocations

When parsing multiple documents:

```rust
let mut buffer = String::with_capacity(4096);
for yaml_file in files {
    buffer.clear();
    read_file(&yaml_file, &mut buffer)?;
    let doc = parse_string(&buffer)?;
    process(doc);
}
```

## Troubleshooting

### Common Issues

#### Parse Errors

**Problem**: "Unexpected character at line X"

**Solution**: Check for:
- Tab characters (use spaces)
- Inconsistent indentation
- Missing quotes around special characters

#### Memory Issues

**Problem**: Out of memory on large documents

**Solution**:
- Enable streaming: `NodeStream::new()`
- Set parser limits: `ParserConfig::with_max_size()`
- Use string interning

#### Type Conversion Failures

**Problem**: `as_str()` returns `None`

**Solution**: Check node type first:
```rust
if node.is_string() {
    let s = node.as_str().unwrap();
}
```

### Getting Help

- GitHub Issues: https://github.com/clockworkengineer/yaml/issues
- Documentation: https://docs.rs/yaml_lib
- Examples: https://github.com/clockworkengineer/yaml/tree/main/examples

## Migration Guide

### From yaml-rust

```rust
// yaml-rust
use yaml_rust::YamlLoader;
let docs = YamlLoader::load_from_str(yaml)?;
let doc = &docs[0];
let value = &doc["key"];

// This library
use yaml_lib::parse_string;
let doc = parse_string(yaml)?;
let value = doc.get_key_safe("key");
```

### From serde_yaml

```rust
// serde_yaml
use serde_yaml::Value;
let value: Value = serde_yaml::from_str(yaml)?;

// This library
use yaml_lib::{parse_string, Node};
let doc: Node = parse_string(yaml)?;
```

## License

Licensed under MIT or Apache-2.0 (your choice).
