# YAML Multi-Document Streams Example

This example demonstrates how to work with YAML files containing multiple documents separated by document markers. Multi-document streams allow you to store related but distinct data structures in a single file.

## What This Example Shows

- Parsing multi-document YAML streams
- Accessing individual documents by index
- Counting documents in a stream
- Using document separators (`---`)
- Using end-of-document markers (`...`)
- Working with different document types in one stream

## Multi-Document Syntax

### Document Separator (`---`)
Marks the start of a new document:
```yaml
---
document: first
name: Document 1
---
document: second
name: Document 2
```

### End Marker (`...`)
Marks the explicit end of a document (optional):
```yaml
---
document: first
...
---
document: second
...
```

## Key Functions

### `get_number_of_documents(node)`
Returns the count of documents in the stream:
```rust
let count = get_number_of_documents(&node);
println!("Found {} documents", count);
```

### `get_document(node, index)`
Retrieves a specific document by zero-based index:
```rust
match get_document(&node, 0) {
    Ok(doc) => println!("{:?}", doc),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Examples Demonstrated

### 1. Basic Multi-Document Stream

Simple demonstration of parsing multiple documents:
```yaml
---
document: first
name: Document 1
---
document: second
name: Document 2
---
document: third
name: Document 3
```

### 2. Accessing Individual Documents

Shows how to access specific documents:
```yaml
---
# User data
user:
  id: 1001
  name: John Doe
---
# User preferences
preferences:
  theme: dark
  language: en
---
# User activity
activity:
  last_login: 2024-01-15
  login_count: 42
```

### 3. Environment Configurations

Multiple environment configs in one file:
```yaml
---
environment: development
database:
  host: localhost
  port: 5432
---
environment: staging
database:
  host: staging-db.example.com
  port: 5432
---
environment: production
database:
  host: prod-db.example.com
  port: 5432
```

### 4. Data Batches

Processing data in batches:
```yaml
---
batch_id: 1
timestamp: 2024-01-15T10:00:00Z
records:
  - id: 1001
    name: Alice Smith
---
batch_id: 2
timestamp: 2024-01-15T10:05:00Z
records:
  - id: 1002
    name: Bob Jones
```

### 5. Mixed Document Types

Different types of documents in one stream:
```yaml
---
# Mapping
type: configuration
settings:
  timeout: 30
---
# Sequence
- item1
- item2
- item3
---
# Scalar
"A simple string"
---
# Null document
...
```

## Usage

```bash
# Run the example
cargo run --example yaml_multi_document

# Or from the example directory
cd examples/yaml_multi_document
cargo run
```

## Output

The example demonstrates:
1. Total document count
2. Individual document access
3. Iterating through all documents
4. Error handling for invalid indices
5. Different document structures

## Real-World Use Cases

### 1. Configuration Management

Store multiple environment configurations:
```yaml
---
# Development
environment: dev
database:
  host: localhost
  debug: true
---
# Production
environment: prod
database:
  host: prod.example.com
  debug: false
```

### 2. Test Fixtures

Multiple test cases in one file:
```yaml
---
test: user_creation
input:
  name: John Doe
  email: john@example.com
expected:
  status: 201
  id: 1001
---
test: user_update
input:
  id: 1001
  name: Jane Doe
expected:
  status: 200
```

### 3. Data Migration

Batch data for migration:
```yaml
---
migration: users
batch: 1
data:
  - id: 1
    name: User 1
---
migration: users
batch: 2
data:
  - id: 2
    name: User 2
```

### 4. Event Logs

Sequential events:
```yaml
---
event: user_login
timestamp: 2024-01-15T10:00:00Z
user_id: 1001
---
event: data_access
timestamp: 2024-01-15T10:05:00Z
user_id: 1001
resource: /api/data
---
event: user_logout
timestamp: 2024-01-15T10:30:00Z
user_id: 1001
```

### 5. API Request/Response Logs

```yaml
---
request:
  method: POST
  url: /api/users
  body:
    name: John Doe
response:
  status: 201
  body:
    id: 1001
    name: John Doe
---
request:
  method: GET
  url: /api/users/1001
response:
  status: 200
  body:
    id: 1001
    name: John Doe
```

## Document Processing Patterns

### Sequential Processing
```rust
let count = get_number_of_documents(&node);
for i in 0..count {
    let doc = get_document(&node, i)?;
    process_document(&doc);
}
```

### Selective Processing
```rust
// Process only specific documents
let config_doc = get_document(&node, 0)?;
let data_doc = get_document(&node, 1)?;

configure_app(&config_doc);
load_data(&data_doc);
```

### Batch Processing
```rust
let count = get_number_of_documents(&node);
let batch_size = 10;

for batch_start in (0..count).step_by(batch_size) {
    let batch_end = (batch_start + batch_size).min(count);
    process_batch(batch_start, batch_end);
}
```

## Best Practices

1. **Use separators consistently** - Always use `---` between documents
2. **Document purpose** - Add comments explaining each document
3. **Logical grouping** - Related documents should be in the same file
4. **Index safely** - Always check document count before accessing
5. **Type consistency** - Consider using similar structures across documents

## Document Markers

### Start Marker (`---`)
- **Required** for multiple documents
- **Optional** for single document (can be omitted)
- Marks the beginning of a document

### End Marker (`...`)
- **Optional** - Explicitly marks document end
- Useful for clarity in streams
- Rarely necessary in practice

### Examples:
```yaml
# Minimal (single document, no markers)
key: value

# Single document with markers
---
key: value
...

# Multiple documents
---
first: document
---
second: document
---
third: document
```

## Error Handling

The example demonstrates handling:
- Invalid document indices
- Parse errors in multi-document streams
- Empty documents
- Missing documents

```rust
match get_document(&node, index) {
    Ok(doc) => println!("Document found"),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Performance Considerations

- All documents are parsed at once (not lazy)
- Document access by index is O(1)
- Memory usage grows with number of documents
- Consider streaming for very large files

## Limitations

- **No cross-document anchors** - Anchors can't reference across documents
- **Memory bound** - All documents loaded into memory
- **Sequential parsing** - Can't skip documents during parse

## Comparison with Alternatives

### Multiple Files
**Pros:**
- Easier to version control
- Can process independently
- Clear separation

**Cons:**
- More files to manage
- Harder to maintain relationships
- More I/O operations

### Single Document Arrays
**Pros:**
- Can use standard YAML arrays
- Simpler structure

**Cons:**
- Less semantic separation
- All items must be same type
- Harder to add metadata per document

### Multi-Document Streams
**Pros:**
- Semantic separation
- Single file
- Different types allowed
- Standard YAML feature

**Cons:**
- All-or-nothing parsing
- Limited tooling support
- Less common pattern

## See Also

- **yaml_parse_and_stringify** - Basic YAML operations
- **yaml_anchors_aliases** - Using anchors (within documents)
- **yaml_fibonacci** - Stateful document processing
