# Safe Node Access Example

This example demonstrates the safe, panic-free API for accessing YAML nodes without risking runtime panics.

## Features Demonstrated

1. **Safe Indexed Access** - `get()` and `get_mut()` for arrays
2. **Safe Key Access** - `get_key()` and `get_key_mut()` for mappings
3. **Type Conversions** - `as_str()`, `as_bool()`, `as_i32()`, `as_f32()`
4. **Type Predicates** - `is_*()` methods for type checking
5. **Collection Views** - `as_slice()`, `as_mapping()`, `keys()`
6. **Error Handling** - Graceful handling of missing or wrong-type values

## Why Use Safe Access?

Traditional indexing (`node[0]` or `node["key"]`) panics if:
- The index is out of bounds
- The key doesn't exist
- The node is the wrong type

Safe access methods return `Option<T>` instead, allowing graceful error handling.

## Usage

```bash
cargo run --example yaml_safe_access
```

## Code Highlights

### Before (Panic-Prone)
```rust
let value = config["database"]["host"];  // Panics if missing!
let port = config["database"]["port"];   // Panics if wrong type!
```

### After (Safe)
```rust
let host = config.get_key("database")
    .and_then(|db| db.get_key("host"))
    .and_then(|h| h.as_str())
    .unwrap_or("localhost");

let port = config.get_key("database")
    .and_then(|db| db.get_key("port"))
    .and_then(|p| p.as_i32())
    .unwrap_or(5432);
```

## Real-World Applications

- **Configuration Loading** - Safely read config files with defaults
- **API Response Parsing** - Handle optional/missing fields gracefully
- **Data Validation** - Check types before processing
- **Defensive Programming** - Eliminate index/key panics
