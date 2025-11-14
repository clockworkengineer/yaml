# Embedded-Safe YAML Example

This example demonstrates panic-free YAML operations for embedded systems.

## Features Demonstrated

1. **Safe Node Access**
   - `get()` and `get_key()` methods that return `Option` instead of panicking
   - Type-safe conversions with `as_i32()`, `as_f32()`, `as_str()`, `as_bool()`
   - Collection queries: `is_sequence()`, `is_mapping()`, `len()`, `is_empty()`

2. **Embedded-Friendly Numerics**
   - Converting to i32/f32 for optimal embedded performance
   - Checking if values fit in i32 range
   - Reporting memory footprint with `size_bytes()`

3. **Node Validation**
   - `NodeValidator` for checking embedded system limits
   - Validates nesting depth, string lengths, collection sizes, anchor counts
   - Returns detailed error information instead of panicking

4. **Safe Parsing**
   - Parse YAML and validate against embedded constraints
   - Extract data safely with Option-based APIs
   - Handle errors gracefully without panics

## Building and Running

```bash
cargo run --manifest-path examples/yaml_embedded_safe/Cargo.toml
```

## Memory Optimization Tips

- Prefer `Int32` over `Integer` (4 bytes vs 8 bytes)
- Use validation to enforce resource limits
- Use safe access methods to avoid panic overhead
- Check `fits_in_i32()` before conversions
