# YAML Fluent API Example

This example demonstrates the new fluent builder API for constructing YAML nodes with a clean, readable syntax.

## Overview

The fluent API provides chainable builder methods that make it easy to construct complex YAML structures without deeply nested function calls or manual vector construction.

## Features Demonstrated

### 1. **Array Building**
Create arrays with a clean, chainable syntax:

```rust
let array = Node::array()
    .push(1)
    .push(2)
    .push(3)
    .build();
```

### 2. **Mapping Building**
Construct mappings with intuitive key-value insertion:

```rust
let config = Node::mapping()
    .insert("host", "localhost")
    .insert("port", 8080)
    .insert("ssl", true)
    .build();
```

### 3. **Nested Structures**
Build complex nested structures with clear hierarchy:

```rust
let doc = Node::mapping()
    .insert("database", Node::mapping()
        .insert("host", "localhost")
        .insert("port", 5432)
        .build())
    .insert("servers", Node::array()
        .push("web1")
        .push("web2")
        .build())
    .build();
```

### 4. **Conditional Building**
Add elements conditionally based on runtime conditions:

```rust
let config = Node::mapping()
    .insert("name", "app")
    .insert_if(debug_mode, "debug", true)
    .insert_opt("optional_key", some_option)
    .build();
```

### 5. **Set Construction**
Create sets with automatic duplicate removal:

```rust
let tags = Node::set()
    .insert("rust")
    .insert("yaml")
    .insert("rust") // automatically ignored
    .build();
```

## Running the Example

```bash
cargo run --example yaml_fluent_api
```

## Builder Methods

### ArrayBuilder

| Method | Description |
|--------|-------------|
| `push(value)` | Add an item to the array |
| `extend(values)` | Add multiple items from an iterator |
| `push_if(condition, value)` | Add item only if condition is true |
| `push_opt(option)` | Add item if Some, skip if None |
| `build()` | Build the final Array node |
| `len()` | Get current number of items |
| `is_empty()` | Check if builder is empty |

### MappingBuilder

| Method | Description |
|--------|-------------|
| `insert(key, value)` | Insert a key-value pair |
| `insert_if(condition, key, value)` | Insert only if condition is true |
| `insert_opt(key, option)` | Insert if Some, skip if None |
| `upsert(key, value)` | Insert or update existing key |
| `build()` | Build the final Mapping node |
| `len()` | Get current number of pairs |
| `is_empty()` | Check if builder is empty |
| `contains_key(key)` | Check if key exists |

### SetBuilder

| Method | Description |
|--------|-------------|
| `insert(value)` | Insert item (duplicates ignored) |
| `extend(values)` | Insert multiple items (duplicates ignored) |
| `build()` | Build the final Set node |
| `len()` | Get current number of unique items |
| `is_empty()` | Check if builder is empty |
| `contains(value)` | Check if value exists |

## Benefits

1. **Readability**: Code reads naturally from left to right, top to bottom
2. **Less Nesting**: Avoids deeply nested function calls
3. **Type Safety**: Fully type-checked with compile-time guarantees
4. **Flexibility**: Mix and match builders for any structure
5. **Conditional Logic**: Built-in support for conditional building
6. **No Overhead**: Zero-cost abstraction - compiles to efficient code

## Comparison: Old vs New

### Old Way (Verbose)
```rust
let config = Node::Mapping(vec![
    (
        Node::from("database"),
        Node::Mapping(vec![
            (Node::from("host"), Node::from("localhost")),
            (Node::from("port"), Node::from(5432)),
        ])
    ),
    (
        Node::from("servers"),
        Node::Array(vec![
            Node::from("web1"),
            Node::from("web2"),
        ])
    ),
]);
```

### New Way (Fluent)
```rust
let config = Node::mapping()
    .insert("database", Node::mapping()
        .insert("host", "localhost")
        .insert("port", 5432)
        .build())
    .insert("servers", Node::array()
        .push("web1")
        .push("web2")
        .build())
    .build();
```

## Use Cases

- **Configuration Files**: Build complex config structures programmatically
- **API Responses**: Construct JSON/YAML responses dynamically
- **Data Transformation**: Convert between formats with readable code
- **Testing**: Create test fixtures with clear structure
- **Code Generation**: Generate YAML programmatically

## Integration

The fluent API works seamlessly with all existing YAML library features:

```rust
use yaml_lib::{stringify, to_json_pretty, BufferDestination};

let config = Node::mapping()
    .insert("key", "value")
    .build();

// Serialize to YAML
let mut yaml_dest = BufferDestination::new();
stringify(&config, &mut yaml_dest)?;

// Serialize to JSON
let mut json_dest = BufferDestination::new();
to_json_pretty(&config, &mut json_dest)?;
```

## See Also

- **yaml_parse_and_stringify** - Basic YAML operations
- **yaml_node_manipulation** - Direct node manipulation
- **yaml_to_json** - Format conversion examples
