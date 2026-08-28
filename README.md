# YAML_lib 🦀

[![Rust](https://img.shields.io/badge/rust-1.88.0+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-Donate-orange.svg)](https://buymeacoffee.com/roberttizz1)

![Tests](https://img.shields.io/badge/internal_tests-1014%20passing-green.svg)
![YAML 1.2](https://img.shields.io/badge/YAML_1.2-100%25%20(402%2F402)-brightgreen.svg)

A comprehensive, high-performance YAML library for Rust with full YAML 1.2 specification compliance (100%, 402/402 tests passing as of Mar 2026), excellent ergonomics, advanced error handling, validation, and extensive format conversion capabilities.

## ✨ Features


### 🏅 **Core YAML Support**
- **100% YAML 1.2 specification compliance** (402/402 tests passing)
- **Unicode-aware parsing** with BOM detection
- **Multi-document streams** support
- **Anchors and aliases** including on mapping keys, with circular reference detection
- **Block and flow syntax** parsing with empty key support
- **Cross-platform line endings** (Unix LF, Windows CRLF)
- **Comments preservation** throughout parsing
- **All scalar types**: strings, integers, floats, booleans, null
- **All collection types**: sequences, mappings, sets


### 🏷️ **Advanced Tag Support**
- **Standard YAML tags**: `!!str`, `!!int`, `!!float`, `!!bool`, `!!null`
- **Collection tags**: `!!seq`, `!!map`, `!!set`
- **Binary data**: `!!binary` with base64 validation
- **Ordered mappings**: `!!omap` for insertion-order preservation
- **Key-value pairs**: `!!pairs` with duplicate key support
- **Merge keys**: `!!merge` for YAML inheritance
- **Numeric bases**: hexadecimal (`!!int:hex`) and octal (`!!int:oct`)
- **Custom tags**: preservation and round-trip support


### 🔄 **Multi-Format Conversion**
- **YAML** ↔ Native format with pretty-printing
- **JSON** ↔ With pretty-printing support
- **XML** ↔ With configurable formatting
- **TOML** ↔ With table structure preservation
- **Bencode** ↔ For BitTorrent applications


### 🚀 **Performance, Safety & Validation**
- **Zero-copy parsing** where possible
- **Memory-efficient** node representation
- **Thread-safe** operations
- **Error recovery** and detailed diagnostics
- **Comprehensive test suite** (1014+ internal tests, 402/402 YAML 1.2 official tests, 100% pass rate)
- **JSON Schema-style validation** for YAML documents
- **Error codes and suggestions** for programmatic error handling


### 🔨 **Fluent Builder API**
- **`ArrayBuilder`** - Chainable API for constructing sequences
- **`MappingBuilder`** - Chainable API for constructing mappings
- **`SetBuilder`** - Chainable API for constructing sets
- **`make_node!` macro** - Concise literal syntax for node construction


### 🔍 **Developer Tools**
- **Node inspection** (`NodeDebugger`, `node_type`, `node_depth`, `print_tree`)
- **Node diffing** (`diff_nodes`, `DiffResult`) for comparing document trees
- **Execution tracing** (`Tracer`, `TraceGuard`) for debugging
- **Debug assertions** (`DebugAssert`, `DebugContext`) with configurable levels


### 🧪 **Testing Infrastructure**
- **Fuzz testing** (`YamlFuzzer`, `fuzz_parse`, `fuzz_roundtrip`) for robustness
- **Property-based testing** (`PropertySuite`) for invariant checking
- **Memory safety auditing** (`SafetyAudit`, `audit_node`) for correctness


### ⚡ **Optimization Utilities**
- **`CapacityHints`** for pre-sizing allocations
- **`FastPathDetector`** for common YAML pattern shortcuts
- **`NodeBuilder`** with allocation reuse
- **`StringInterner` / `SimpleInterner`** for string deduplication
- **`InternedString`** / `CommonStrings`** for zero-cost key sharing

## 🛡️ Error Handling & Validation

### Centralized Error Handling (Contributor Note)

All parser and lexer error messages must use the centralized helpers in `parser/document/error_builder.rs` (e.g., `syntax_error`, `structure_error`, `limit_error`, `forbidden_error`).

**Do not return raw error strings.**

This ensures all errors are consistent, include context, and are easy to maintain. See the module-level docs in `error_builder.rs` for usage examples and extension guidelines.


### Error Handling
- **Error codes (E001-E015)** for programmatic handling
- **Intelligent suggestions** for fixing common mistakes
- **Error recovery strategies** to continue parsing after errors
- **Enhanced error context** with source spans and snippets
- **Multi-error collection** for batch reporting

### Validation
- **JSON Schema-style validation** for YAML documents
- **Type checking, constraint validation, and schema enforcement**
- **Built-in validators**: type, range, length, pattern, enum, required, custom
- **Comprehensive validation examples** in `examples/yaml_validation/`

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
yaml_lib = "0.2.1"
```

## 🚀 Quick Start

### Basic Parsing

```rust
use yaml_lib::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse YAML from a string (convenience function)
    let yaml_str = r#"
    name: "YAML Library"
    version: 1.0
    features:
      - parsing
      - serialization
      - multi-format
    "#;
    
    let document = parse_string(yaml_str)?;
    println!("{:#?}", document);
    
    // Or parse from a file directly
    // let document = parse_file("config.yaml")?;
    
    // Or use the lower-level source-based API
    let mut source = BufferSource::new(yaml_str.as_bytes());
    let document = parse(&mut source)?;
    println!("{:#?}", document);
    Ok(())
}
```

### Working with Nodes

```rust
use yaml_lib::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create nodes programmatically
    let config = make_node! {
        "database" => {
            "host" => "localhost",
            "port" => 5432,
            "ssl" => true
        },
        "servers" => ["web1", "web2", "web3"]
    };
    
    // Convert to different formats
    let mut buffer = BufferDestination::new();
    
    // To YAML
    stringify(&config, &mut buffer)?;
    println!("YAML:\n{}", buffer.to_string());
    buffer.clear();
    
    // To JSON
    to_json_pretty(&config, &mut buffer)?;
    println!("JSON:\n{}", buffer.to_string());
    buffer.clear();
    
    // To XML
    to_xml_pretty(&config, &mut buffer)?;
    println!("XML:\n{}", buffer.to_string());
    
    Ok(())
}
```

### File Operations

```rust
use yaml_lib::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read from file with automatic encoding detection
    let content = read_file_to_string("config.yaml")?;
    let mut source = BufferSource::new(content.as_bytes());
    let document = parse(&mut source)?;
    
    // Write to different formats
    let mut json_dest = FileDestination::new("output.json")?;
    to_json_pretty(&document, &mut json_dest)?;
    
    let mut xml_dest = FileDestination::new("output.xml")?;
    to_xml_pretty(&document, &mut xml_dest)?;
    
    Ok(())
}
```

### Advanced Features

```rust
use yaml_lib::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Working with tags and anchors
    let yaml_with_tags = r#"
    binary_data: !!binary |
      R0lGODlhDAAMAIQAAP//9/X17unp5WZmZgAAAOfn515eXvPz7Y6OjuDg4J+fn5
      OTk6enp56enmlpaWNjY6Ojo4SEhP/++f/++f/++f/++f/++f/++f/++f/++f/++Q==
    
    ordered_map: !!omap
      - first: 1
      - second: 2
      - third: 3
    
    config: &default_config
      timeout: 30
      retries: 3
    
    development:
      <<: *default_config
      debug: true
    
    production:
      <<: *default_config
      debug: false
    "#;
    
    let mut source = BufferSource::new(yaml_with_tags.as_bytes());
    let document = parse(&mut source)?;
    
    // Access specific documents in multi-doc streams
    let base_doc = get_document(&document, 0)?;
    
    Ok(())
}
```

## 📚 API Reference

### Core Types

- **`Node`** - The fundamental data structure representing any YAML value
- **`Numeric`** - Enum for integer and floating-point numbers
- **`BufferSource`/`FileSource`** - Input sources for parsing
- **`BufferDestination`/`FileDestination`** - Output destinations for serialization

### Key Functions

| Function | Description |
|----------|-------------|
| `parse()` | Parse YAML from any source into a Node tree |
| `parse_string()` | Parse YAML from an in-memory `&str` (default config) |
| `parse_file()` | Parse YAML from a file path (default config) |
| `parse_with_config()` | Parse YAML from a string with a `ParserConfig` |
| `parse_string_with_recovery()` | Parse with error recovery, returns `(Node, Vec<YamlError>)` |
| `stringify()` | Convert Node tree back to YAML format |
| `to_json()` / `to_json_pretty()` | Convert to JSON format |
| `to_xml()` / `to_xml_pretty()` | Convert to XML format |
| `to_toml()` / `to_toml_pretty()` | Convert to TOML format |
| `to_bencode()` | Convert to Bencode format |
| `make_node!` | Macro for creating nodes with literal syntax |
| `make_set()` | Create set nodes with duplicate removal |

### File Utilities

| Function | Description |
|----------|-------------|
| `read_file_to_string()` | Read file with automatic encoding detection |
| `write_file_from_string()` | Write file with specified encoding |
| `detect_format()` | Detect Unicode format from BOM |

## 🎯 Examples

The repository includes 23 comprehensive examples:

| Example | Description |
|---------|-------------|
| [yaml_parse_and_stringify](../examples/yaml_parse_and_stringify/) | Basic parsing and serialization |
| [yaml_to_json](../examples/yaml_to_json/) | YAML → JSON conversion |
| [yaml_to_xml](../examples/yaml_to_xml/) | YAML → XML conversion |
| [yaml_to_toml](../examples/yaml_to_toml/) | YAML → TOML conversion |
| [yaml_to_bencode](../examples/yaml_to_bencode/) | YAML → Bencode conversion |
| [yaml_anchors_aliases](../examples/yaml_anchors_aliases/) | Anchors, aliases, and merge keys |
| [yaml_advanced_tags](../examples/yaml_advanced_tags/) | Custom and standard tag handling |
| [yaml_multi_document](../examples/yaml_multi_document/) | Multi-document YAML streams |
| [yaml_node_manipulation](../examples/yaml_node_manipulation/) | Programmatic tree building |
| [yaml_fluent_api](../examples/yaml_fluent_api/) | Fluent builder API |
| [yaml_error_handling](../examples/yaml_error_handling/) | Error codes and recovery |
| [yaml_validation](../examples/yaml_validation/) | JSON Schema-style validation |
| [yaml_safe_access](../examples/yaml_safe_access/) | Nil-safe deep path access |
| [yaml_tree_traversal](../examples/yaml_tree_traversal/) | Iterators and traversal orders |
| [yaml_streaming](../examples/yaml_streaming/) | Streaming serialization |
| [yaml_string_interning](../examples/yaml_string_interning/) | String deduplication |
| [yaml_performance](../examples/yaml_performance/) | Benchmark helpers |
| [yaml_performance_opts](../examples/yaml_performance_opts/) | Optimization strategies |
| [yaml_embedded_systems](../examples/yaml_embedded_systems/) | `no_std` / embedded target |
| [yaml_embedded_safe](../examples/yaml_embedded_safe/) | Safe embedded parsing |
| [yaml_fibonacci](../examples/yaml_fibonacci/) | Algorithmic YAML generation |
| [yaml_comprehensive](../examples/yaml_comprehensive/) | End-to-end showcase |
| [yaml_utility_lib](../examples/yaml_utility_lib/) | Library usage patterns |

Run any example:
```bash
cargo run -p yaml_parse_and_stringify
cargo run -p yaml_to_json
cargo run -p yaml_validation
```

## 🧪 Testing

The library includes an extensive test suite with 1014+ tests covering:

- **Basic parsing** - All YAML constructs and edge cases
- **Document structure** - Multi-document streams, markers, directives
- **Tag coercion** - All standard and custom tags
- **Error handling** - Malformed YAML and recovery
- **File parsing** - Different encodings and formats
- **Nested structures** - Deep nesting and complex documents
- **Flow syntax** - Inline sequences and mappings
- **Set operations** - Unique collections and operations
- **Anchor/alias resolution** - Including circular reference detection
- **Directive handling** - `%YAML` and `%TAG` directives
- **Official YAML test suite** - 402/402 tests (100% compliance)

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test basic_parsing
cargo test tag_coercion
cargo test error_handling

# Run with output
cargo test -- --nocapture
```

## 🪵 Debug Logging

Debug logging is opt-in and disabled by default to avoid overhead. Enable it with the `debug-trace` feature and a logger (e.g., `env_logger`).

### Enable in tests (Windows PowerShell)

```powershell
# Enable logging for this session
$env:RUST_LOG = 'yaml_lib=debug'

# Promote token-stream internals to debug without global trace
$env:YAML_TRACE_TOKENS = '1'

# Run tests with logging enabled
cargo test -p yaml_lib --features debug-trace -- --nocapture

# For very verbose internals, use full trace instead of YAML_TRACE_TOKENS
# $env:RUST_LOG = 'yaml_lib=trace'
```

### Enable in binaries/examples

Add a logger init (once) in your `main`:

```rust
fn main() {
  // Initialize any `log` compatible logger
  let _ = env_logger::try_init();
  // ... your code
}
```

Run with the feature and env vars as needed:

```powershell
$env:RUST_LOG = 'yaml_lib=debug'; $env:YAML_TRACE_TOKENS = '1'
cargo run --features debug-trace
```

Notes:
- `yaml_lib=debug` shows high-level parser decisions; token-stream stays quiet unless `YAML_TRACE_TOKENS` is set.
- Set `yaml_lib=trace` for maximum verbosity (may be very chatty).
- Feature-gating ensures zero overhead when `debug-trace` is not enabled.

## 🔧 Performance

YAML_lib is designed for performance:

- **Lazy parsing** - Only parse what you need
- **Memory efficiency** - Minimal allocations and copying
- **Zero-copy strings** - Where possible, reference original data
- **Optimized algorithms** - Efficient parsing and serialization
- **Minimal dependencies** - Only `rand` for testing utilities

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

```bash
git clone https://github.com/clockworkengineer/yaml.git
cd yaml
cargo build
cargo test
```

### Code Guidelines

- Follow standard Rust formatting (`cargo fmt`)
- Ensure all tests pass (`cargo test`)
- Add tests for new functionality
- Update documentation as needed

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ☕ Support

If you find this library useful, feel free to support its development!

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-Donate-orange.svg)](https://buymeacoffee.com/roberttizz1)

You can [buy me a coffee here](https://buymeacoffee.com/roberttizz1).

## 🙏 Acknowledgments

- **YAML 1.2 Specification** - [yaml.org](https://yaml.org/spec/1.2.2/)
- **Rust Community** - For excellent tooling and ecosystem
- **Contributors** - Everyone who helped improve this library

## 📊 Project Stats

- **Language**: Rust 🦀
- **Minimum Rust Version**: 1.88.0
- **Lines of Code**: ~15,000+
- **Internal Tests**: 1014+ passing
- **Official YAML 1.2 Tests**: 402/402 (100%)
- **Documentation**: Comprehensive inline docs and doc-tests
- **Examples**: 23 comprehensive examples

---

**Made with ❤️ and 🦀 by the YAML_lib team**

*For questions, issues, or contributions, please visit our [GitHub repository](https://github.com/clockworkengineer/yaml).*