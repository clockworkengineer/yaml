# YAML Fibonacci Sequence Generator Example

This example demonstrates advanced YAML Node manipulation by implementing a Fibonacci sequence generator that stores its state in a YAML file.

## What This Example Shows

- Creating and manipulating `Node` structures programmatically
- Working with YAML arrays (`Node::Array`)
- File-based state persistence with YAML
- Reading and writing YAML with `FileSource` and `FileDestination`
- Pattern matching on Node types
- Arithmetic operations with `Node::Number`
- Incremental data generation

## How It Works

The example:
1. Checks if `fibonacci.yaml` exists
2. If it exists, reads and parses the current sequence
3. If it doesn't exist, initializes with `[1, 1]`
4. Calculates the next Fibonacci number by adding the last two numbers
5. Appends the new number to the sequence
6. Writes the updated sequence back to the file

Each time you run the program, it adds one more Fibonacci number to the sequence.

## Usage

```bash
# Run the example from the project root
cargo run --example yaml_fibonacci

# Run it multiple times to grow the sequence
cargo run --example yaml_fibonacci
cargo run --example yaml_fibonacci
cargo run --example yaml_fibonacci
```

## Example Progression

### First Run - Creates Initial File
```yaml
# fibonacci.yaml
- 1
- 1
```

### Second Run
```yaml
# fibonacci.yaml
- 1
- 1
- 2
```

### Third Run
```yaml
# fibonacci.yaml
- 1
- 1
- 2
- 3
```

### After Several Runs
```yaml
# fibonacci.yaml
- 1
- 1
- 2
- 3
- 5
- 8
- 13
- 21
- 34
- 55
- 89
- 144
```

## Key Concepts Demonstrated

### 1. Node Creation
```rust
// Creating initial sequence
Node::Array(vec![
    Node::Number(Numeric::Integer(1)),
    Node::Number(Numeric::Integer(1)),
])
```

### 2. Node Pattern Matching
```rust
// Extracting values from nodes
match (&items[items.len() - 2], &items[items.len() - 1]) {
    (Node::Number(Numeric::Integer(a)), Node::Number(Numeric::Integer(b))) => {
        // Work with the integer values
    }
    _ => {}
}
```

### 3. Node Manipulation
```rust
// Adding new elements to array
if let Node::Array(items) = sequence {
    items.push(Node::Number(Numeric::Integer(sum)));
}
```

### 4. Overflow Protection
```rust
// Using checked arithmetic to prevent overflow
if let Some(sum) = a.checked_add(*b) {
    items.push(Node::Number(Numeric::Integer(sum)));
}
```

## Functions Breakdown

### `read_sequence(file_path)`
Reads the Fibonacci sequence from a YAML file. If the file doesn't exist, returns an initial sequence `[1, 1]`.

**Returns:** `Result<Node, String>`

### `add_next(sequence)`
Calculates and appends the next Fibonacci number to the sequence by summing the last two numbers.

**Parameters:** 
- `sequence: &mut Node` - The sequence to modify

### `write_sequence(file_path, sequence)`
Saves the Fibonacci sequence back to the YAML file.

**Returns:** `Result<(), String>`

## Key Functions Used

- **`FileSource::new(path)`** - Opens a YAML file for reading
- **`parse(&mut source)`** - Parses YAML into a Node tree
- **`FileDestination::new(path)`** - Creates a destination file for writing
- **`stringify(&node, &mut destination)`** - Converts Node tree back to YAML

## Error Handling

The example handles several error conditions:
- **Missing file** - Initializes with default sequence
- **Invalid format** - Reports errors if file isn't a valid sequence
- **Parse errors** - Catches and reports YAML syntax errors
- **Write errors** - Reports file system errors
- **Integer overflow** - Uses checked arithmetic to prevent panics

## Use Cases

This pattern is useful for:
- **State persistence** - Maintaining application state in YAML
- **Incremental data** - Building datasets over time
- **Configuration evolution** - Tracking configuration changes
- **Data collection** - Accumulating data from multiple runs
- **Simple databases** - File-based data storage

## Extending This Example

You could extend this to:
- Generate other mathematical sequences (primes, triangular numbers)
- Track multiple sequences in a single YAML file
- Add sequence analysis (sum, average, patterns)
- Implement sequence limits or truncation
- Add timestamps for each generated number

## Educational Value

This example teaches:
- **Node type system** - Understanding how YAML structures map to Nodes
- **File I/O patterns** - Reading, modifying, and writing YAML
- **Safe arithmetic** - Preventing integer overflow
- **State management** - Persisting program state across runs
- **Error handling** - Gracefully handling various error conditions

## See Also

- **yaml_parse_and_stringify** - Basic YAML I/O operations
- **yaml_utility_lib** - Utility functions for YAML operations
- **Node manipulation examples** - More advanced Node operations
