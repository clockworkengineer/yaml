# YAML Anchors and Aliases Example

This example demonstrates how to use YAML anchors and aliases to reuse configuration blocks and create inheritance patterns. This is one of YAML's most powerful features for avoiding duplication.

## What This Example Shows

- **Anchors** (`&name`) - Marking a node for reuse
- **Aliases** (`*name`) - Referencing an anchored node
- **Merge keys** (`<<:`) - Inheriting and overriding configuration
- **Nested anchors** - Complex reference patterns
- **Multiple references** - Using the same anchor in many places

## Core Concepts

### Anchors (`&`)
An anchor marks a node so it can be referenced elsewhere:
```yaml
default_config: &defaults
  timeout: 30
  retries: 3
```

### Aliases (`*`)
An alias references a previously anchored node:
```yaml
service_a: *defaults  # Gets a copy of default_config
```

### Merge Keys (`<<:`)
The merge key inherits properties from an anchor:
```yaml
production:
  <<: *defaults      # Inherit all properties
  timeout: 60        # Override specific properties
```

## Examples Demonstrated

### 1. Basic Anchors and Aliases
Shows simple node reuse:
```yaml
default_config: &defaults
  timeout: 30
  retries: 3
  log_level: info

service_a: *defaults
service_b: *defaults
```

Both services get identical configuration.

### 2. Configuration Inheritance
Demonstrates using merge keys for DRY configuration:
```yaml
base: &base_config
  timeout: 30
  retries: 3
  log_level: info

development:
  <<: *base_config    # Inherit everything
  log_level: debug    # Override log level
  debug_mode: true    # Add new property

production:
  <<: *base_config
  timeout: 60
  log_level: warn
```

Each environment inherits base settings but customizes as needed.

### 3. Nested Anchors
Shows anchors within anchors:
```yaml
database_config: &db
  host: localhost
  port: 5432
  credentials: &creds
    username: admin
    password: secret

primary_db: *db              # Reuses entire config

backup_db:
  host: backup.example.com
  credentials: *creds        # Reuses just credentials
```

You can anchor at any level and reference selectively.

### 4. Multiple References
Demonstrates one anchor used many times:
```yaml
common_features: &features
  - authentication
  - logging
  - metrics

services:
  web_api:
    features: *features
  admin_api:
    features: *features
  metrics_api:
    features: *features
```

Changes to `common_features` would affect all services.

## Usage

```bash
# Run the example
cargo run --example yaml_anchors_aliases

# Or from the example directory
cd examples/yaml_anchors_aliases
cargo run
```

## Output

The example shows:
1. Original YAML with anchors
2. Parsed and resolved node structure
3. Stringified output showing resolved values
4. How the library handles each pattern

## Key Concepts

### Anchor Resolution
When the YAML parser encounters an alias (`*name`):
1. It looks up the anchor with that name
2. Copies the anchored node's structure
3. Inserts the copy at the alias location

### Merge Key Resolution
When the parser encounters `<<: *name`:
1. It takes all key-value pairs from the anchored mapping
2. Inserts them into the current mapping
3. Later keys override inherited values

### Memory Efficiency
The library resolves aliases during parsing:
- Anchors are expanded into the tree structure
- No runtime alias resolution needed
- Final Node tree has no anchor/alias references

## Real-World Use Cases

### Docker Compose Files
```yaml
x-common-service: &common
  restart: unless-stopped
  networks:
    - app-network
  logging:
    driver: json-file
    options:
      max-size: "10m"

services:
  web:
    <<: *common
    image: web:latest
    
  api:
    <<: *common
    image: api:latest
```

### Kubernetes Configuration
```yaml
defaults: &defaults
  resources:
    requests:
      memory: "64Mi"
      cpu: "250m"
    limits:
      memory: "128Mi"
      cpu: "500m"

containers:
  - name: app
    <<: *defaults
    image: myapp:1.0
  - name: sidecar
    <<: *defaults
    image: sidecar:1.0
```

### Application Configuration
```yaml
database: &db_config
  pool_size: 10
  timeout: 30
  ssl_mode: require

environments:
  development:
    database:
      <<: *db_config
      host: localhost
      
  production:
    database:
      <<: *db_config
      host: prod.example.com
      pool_size: 50
```

## Benefits

1. **DRY (Don't Repeat Yourself)** - Define once, use many times
2. **Consistency** - Same configuration across multiple locations
3. **Maintainability** - Update in one place, affects all references
4. **Inheritance** - Base configurations with environment-specific overrides
5. **Clarity** - Makes relationships between configurations explicit

## Limitations

- Anchors must be defined before they're referenced
- Can't reference anchors across documents in multi-document streams
- Circular references are detected and prevented
- Merge keys work only with mappings, not sequences

## Advanced Patterns

### Multiple Merge Keys
```yaml
<<: [ *base1, *base2, *base3 ]
```
Merges multiple anchored mappings (later ones override earlier).

### Anchored Sequences
```yaml
common_ports: &ports
  - 80
  - 443

server:
  ports: *ports
```

### Complex Inheritance
```yaml
base: &base
  setting1: value1

middle: &middle
  <<: *base
  setting2: value2

final:
  <<: *middle
  setting3: value3
```

## Error Handling

The example demonstrates handling:
- Undefined anchor references
- Circular reference detection
- Parse errors with anchors
- Invalid merge key usage

## See Also

- **yaml_advanced_tags** - Other YAML advanced features
- **yaml_parse_and_stringify** - Basic YAML operations
- **yaml_multi_document** - Multi-document with anchors
