# YAML Node Manipulation Example

This example demonstrates how to programmatically create and manipulate YAML nodes using the library's API. Instead of parsing YAML text, you build data structures directly in code.

## What This Example Shows

- **Manual node creation** - Using constructors directly
- **`make_node!` macro** - Convenient syntax for creating nodes
- **`make_set!` function** - Creating sets with duplicate removal
- **Node modification** - Updating existing structures
- **Complex nesting** - Building multi-level hierarchies
- **Practical examples** - Real-world configuration building

## Node Types

The library provides these core node types:

```rust
pub enum Node {
    String(String),           // Text values
    Number(Numeric),          // Integer or Float
    Boolean(bool),            // true or false
    Null,                     // null value
    Array(Vec<Node>),         // Sequences
    Object(HashMap<String, Node>), // Mappings
    // ... and other specialized types
}
```

## Creation Methods

### 1. Manual Construction

Direct instantiation using constructors:

```rust
// Scalars
let s = Node::String("Hello".to_string());
let i = Node::Number(Numeric::Integer(42));
let f = Node::Number(Numeric::Float(3.14));
let b = Node::Boolean(true);
let n = Node::Null;

// Collections
let arr = Node::Array(vec![
    Node::String("one".to_string()),
    Node::String("two".to_string()),
]);

let mut map = HashMap::new();
map.insert("key".to_string(), Node::String("value".to_string()));
let obj = Node::Object(map);
```

### 2. Using `make_node!` Macro

Convenient syntax for common patterns:

```rust
// Scalars
let name = make_node!("Alice");
let age = make_node!(30);
let score = make_node!(95.5);
let active = make_node!(true);

// Arrays
let colors = make_node!(["red", "green", "blue"]);
let numbers = make_node!([1, 2, 3, 4, 5]);

// Objects
let person = make_node!({
    "name" => "Bob",
    "age" => 35,
    "email" => "bob@example.com"
});

// Nested structures
let config = make_node!({
    "database" => {
        "host" => "localhost",
        "port" => 5432
    },
    "servers" => ["web1", "web2", "web3"]
});
```

### 3. Creating Sets

Use `make_set` for unique collections:

```rust
let tags = make_set(vec![
    make_node!("rust"),
    make_node!("yaml"),
    make_node!("rust"),  // Duplicate removed
]);
```

## Usage

```bash
# Run the example
cargo run --example yaml_node_manipulation

# Or from the example directory
cd examples/yaml_node_manipulation
cargo run
```

## Examples Demonstrated

### 1. Manual Node Creation

Shows explicit construction of all node types.

### 2. make_node! Macro

Demonstrates the convenient macro syntax for rapid development.

### 3. Complex Structures

Builds a complete e-commerce order structure:
```rust
let order = make_node!({
    "order_id" => "ORD-2024-001",
    "customer" => {
        "name" => "Jane Doe",
        "address" => {
            "street" => "123 Main St",
            "city" => "Springfield"
        }
    },
    "items" => [
        {
            "product_id" => "PROD-001",
            "name" => "Widget",
            "price" => 29.99
        }
    ]
});
```

### 4. Node Modification

Shows how to modify existing nodes:
```rust
if let Node::Object(ref mut map) = config {
    // Update value
    map.insert("version".to_string(), make_node!("2.0.0"));
    
    // Add new field
    map.insert("build_date".to_string(), make_node!("2024-01-15"));
    
    // Modify nested array
    if let Some(Node::Array(features)) = map.get_mut("features") {
        features.push(make_node!("premium"));
    }
}
```

### 5. Set Creation

Demonstrates automatic duplicate removal:
```rust
let tags = make_set(vec![
    make_node!("rust"),
    make_node!("yaml"),
    make_node!("rust"),  // Removed
    make_node!("parsing"),
]);
```

### 6. Application Configuration

Builds a complete, production-ready app configuration programmatically.

## Real-World Use Cases

### Dynamic Configuration

```rust
fn create_env_config(env: &str) -> Node {
    let (host, port, debug) = match env {
        "development" => ("localhost", 3000, true),
        "staging" => ("staging.example.com", 8080, false),
        "production" => ("prod.example.com", 443, false),
        _ => ("localhost", 3000, true),
    };
    
    make_node!({
        "environment" => env,
        "server" => {
            "host" => host,
            "port" => port
        },
        "debug" => debug
    })
}
```

### Configuration Merging

```rust
fn merge_configs(base: &Node, override_node: &Node) -> Node {
    // Implementation to merge two configuration nodes
    // Override values from override_node into base
    // ...
}
```

### Data Transformation

```rust
fn transform_data(input: Vec<User>) -> Node {
    let users = input.into_iter()
        .map(|user| make_node!({
            "id" => user.id,
            "name" => user.name,
            "email" => user.email
        }))
        .collect::<Vec<_>>();
    
    make_node!({
        "users" => users,
        "count" => users.len() as i64
    })
}
```

### API Response Building

```rust
fn build_response(data: Data, status: &str) -> Node {
    make_node!({
        "status" => status,
        "timestamp" => chrono::Utc::now().to_rfc3339(),
        "data" => {
            "id" => data.id,
            "values" => data.values
        },
        "metadata" => {
            "version" => "1.0",
            "request_id" => generate_request_id()
        }
    })
}
```

### Test Data Generation

```rust
fn generate_test_users(count: usize) -> Node {
    let users = (0..count)
        .map(|i| make_node!({
            "id" => i as i64 + 1000,
            "name" => format!("User {}", i),
            "email" => format!("user{}@example.com", i),
            "active" => true
        }))
        .collect::<Vec<_>>();
    
    Node::Array(users)
}
```

## Modification Patterns

### Adding Fields

```rust
if let Node::Object(map) = &mut node {
    map.insert("new_field".to_string(), make_node!("value"));
}
```

### Updating Values

```rust
if let Node::Object(map) = &mut node {
    if let Some(value) = map.get_mut("field") {
        *value = make_node!("new_value");
    }
}
```

### Removing Fields

```rust
if let Node::Object(map) = &mut node {
    map.remove("field_to_remove");
}
```

### Appending to Arrays

```rust
if let Node::Array(arr) = &mut node {
    arr.push(make_node!("new_item"));
}
```

### Filtering Arrays

```rust
if let Node::Array(arr) = &mut node {
    arr.retain(|item| {
        // Keep only items matching criteria
        matches!(item, Node::Boolean(true))
    });
}
```

## Best Practices

1. **Use `make_node!` for clarity** - More readable than manual construction
2. **Pattern match safely** - Always handle unexpected node types
3. **Validate modifications** - Ensure structure remains valid
4. **Document structure** - Comment expected node shapes
5. **Use type wrappers** - Create domain types around nodes

## Type Safety Tips

### Wrapping Nodes in Domain Types

```rust
struct Config {
    node: Node,
}

impl Config {
    fn new() -> Self {
        Config {
            node: make_node!({
                "version" => "1.0.0",
                "settings" => {}
            })
        }
    }
    
    fn get_version(&self) -> Option<&str> {
        if let Node::Object(map) = &self.node {
            if let Some(Node::String(v)) = map.get("version") {
                return Some(v);
            }
        }
        None
    }
}
```

### Builder Pattern

```rust
struct ConfigBuilder {
    node: Node,
}

impl ConfigBuilder {
    fn new() -> Self {
        ConfigBuilder {
            node: make_node!({}),
        }
    }
    
    fn with_database(mut self, host: &str, port: i64) -> Self {
        if let Node::Object(map) = &mut self.node {
            map.insert("database".to_string(), make_node!({
                "host" => host,
                "port" => port
            }));
        }
        self
    }
    
    fn build(self) -> Node {
        self.node
    }
}
```

## Performance Considerations

- **`make_node!` is not zero-cost** - Convenience over performance
- **Direct construction is faster** - For performance-critical code
- **Cloning nodes** - Nodes implement Clone but it's not always cheap
- **Large structures** - Consider streaming for very large data

## Integration with Serialization

After building nodes, you can serialize to any format:

```rust
let config = make_node!({ /* ... */ });

// To YAML
stringify(&config, &mut yaml_dest)?;

// To JSON
to_json_pretty(&config, &mut json_dest)?;

// To XML
to_xml_pretty(&config, &mut xml_dest)?;
```

## See Also

- **yaml_parse_and_stringify** - Parsing YAML into nodes
- **yaml_fibonacci** - Modifying nodes from files
- **yaml_advanced_tags** - Working with tagged nodes
