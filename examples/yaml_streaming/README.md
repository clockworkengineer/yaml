# YAML Streaming and Iterator Example

This example demonstrates streaming and iterator support for efficient YAML processing, especially useful for large documents.

## Features

1. **Node Iteration**
   - Depth-first traversal (pre-order)
   - Breadth-first traversal (level-order)
   - Memory-efficient iteration

2. **Stream Operations**
   - Filter: Select nodes matching criteria
   - Map: Transform nodes
   - Fold: Reduce/aggregate node data

3. **Path-Based Access**
   - Navigate nested structures with paths
   - Support for both key and index access
   - Type-safe path construction

4. **Node Collection**
   - Collect all strings in a document
   - Collect all numbers in a document
   - Filter nodes by type or content

5. **Large Document Processing**
   - Process documents without loading everything into memory
   - Efficient statistics calculation
   - Stream-based searching

## Running

```bash
cargo run --example yaml_streaming
```

Or from the example directory:
```bash
cd examples/yaml_streaming
cargo run
```

## Examples Included

### 1. Basic Iteration
Iterate through all nodes in a document:
```rust
for node in tree.iter_depth_first() {
    match node {
        Node::Str(s, _, _) => println!("String: {}", s),
        Node::Number(num) => println!("Number: {:?}", num),
        _ => {}
    }
}
```

### 2. Traversal Orders
Compare depth-first vs breadth-first:
```rust
// Depth-first: visits children before siblings
tree.iter_depth_first()

// Breadth-first: visits all siblings before children
tree.iter_breadth_first()
```

### 3. Filtering and Searching
Find nodes matching criteria:
```rust
// Find first matching node
let admin = tree.find_node(|n| {
    matches!(n, Node::Str(s, _, _) if s == "admin")
});

// Filter all matching nodes
let all_admins = tree.filter_nodes(|n| {
    matches!(n, Node::Str(s, _, _) if s == "admin")
});
```

### 4. Stream Operations
Process nodes with functional operations:
```rust
let stream = NodeStream::new(&tree);

// Filter
let expensive = stream.filter(|n| {
    matches!(n, Node::Number(Float(price)) if *price > 20.0)
});

// Map
let prices = stream.map(|n| match n {
    Node::Number(Float(price)) => Some(*price),
    _ => None,
});

// Fold/Reduce
let total = stream.fold(0.0, |acc, n| match n {
    Node::Number(Float(price)) => acc + price,
    _ => acc,
});
```

### 5. Path-Based Access
Access nested values using paths:
```rust
let mut path = NodePath::new();
path.push("config");
path.push("database");
path.push("host");

if let Some(node) = path.get(&tree) {
    println!("Found: {:?}", node);
}

// Array index access
path.push(0usize);  // First element
```

### 6. Collecting Types
Collect all values of specific types:
```rust
// All strings
let strings = tree.collect_strings();

// All integers
let numbers = tree.collect_numbers();

// Count total nodes
let count = tree.count_nodes();
```

### 7. Large Document Processing
Efficiently process large YAML files:
```rust
// Parse large document
let tree = parse(&mut source)?;

// Count nodes without building intermediate collections
let count = tree.count_nodes();

// Find items efficiently
let high_value = NodeStream::new(&tree)
    .filter(|n| matches!(n, Node::Number(Integer(v)) if *v > 5000))
    .count();

// Calculate statistics
let (sum, count) = NodeStream::new(&tree)
    .fold((0, 0), |(sum, count), n| {
        match n {
            Node::Number(Integer(v)) => (sum + v, count + 1),
            _ => (sum, count),
        }
    });
```

## Performance Benefits

- **Memory Efficient**: Iterators don't duplicate data
- **Lazy Evaluation**: Only processes nodes you access
- **No Intermediate Collections**: Stream operations are chained
- **Large File Support**: Process huge documents incrementally

## Use Cases

1. **Configuration Validation**: Search for specific settings
2. **Data Extraction**: Collect all values of a type
3. **Statistics**: Calculate sums, averages, counts
4. **Filtering**: Extract subset of data
5. **Transformation**: Convert or map node values
6. **Large Logs**: Process YAML logs without loading all into memory

## API Summary

### NodeIteratorExt Trait
- `iter_depth_first()` - Depth-first iterator
- `iter_breadth_first()` - Breadth-first iterator
- `count_nodes()` - Count all nodes
- `find_node(predicate)` - Find first match
- `filter_nodes(predicate)` - Filter all matches
- `collect_strings()` - Get all strings
- `collect_numbers()` - Get all integers

### NodeStream
- `new(node)` - Create stream from node
- `filter(predicate)` - Filter nodes
- `map(mapper)` - Transform nodes
- `fold(init, folder)` - Reduce to single value
- `count()` - Count filtered nodes
- `collect()` - Gather into Vec

### NodePath
- `new()` - Create empty path
- `push(segment)` - Add key or index
- `get(node)` - Access node at path
- `from_segments(vec)` - Build from segments

## See Also

- **yaml_performance_opts** - Performance optimization techniques
- **yaml_tree_traversal** - Tree traversal patterns
- **yaml_node_manipulation** - Node creation and modification
