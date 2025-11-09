# YAML to TOML Converter Example

This example demonstrates how to convert YAML files to TOML format using the YAML library's TOML serialization capabilities.

## What This Example Shows

- Reading YAML files using `FileSource`
- Parsing YAML content into Node structures
- Converting Node trees to TOML format with `to_toml()`
- Writing TOML output to files using `FileDestination`
- Batch processing multiple files

## How It Works

The example:
1. Scans the `files/` directory for all `.yaml` files
2. For each YAML file:
   - Opens and parses the YAML content
   - Converts the parsed Node tree to TOML format
   - Writes the TOML output to a new file with `.toml` extension

This is useful for:
- **Rust ecosystem integration** - TOML is the standard for Cargo.toml and Rust configs
- **Configuration management** - Converting between config formats
- **Project migration** - Moving from YAML to TOML configurations
- **Format standardization** - Using TOML's explicit structure

## Usage

```bash
# Run the example from the project root
cargo run --example yaml_to_toml

# Or run from the examples directory
cd examples/yaml_to_toml
cargo run
```

## Example Input/Output

### Input (`config.yaml`)
```yaml
package:
  name: my-app
  version: 1.0.0
  authors:
    - John Doe
    - Jane Smith

dependencies:
  serde: 1.0
  tokio:
    version: 1.0
    features:
      - full
      - macros

server:
  host: localhost
  port: 8080
  ssl: true
```

### Output (`config.toml`)
```toml
[package]
name = "my-app"
version = "1.0.0"
authors = ["John Doe", "Jane Smith"]

[dependencies]
serde = "1.0"

[dependencies.tokio]
version = "1.0"
features = ["full", "macros"]

[server]
host = "localhost"
port = 8080
ssl = true
```

## TOML Formatting

The library provides two TOML output options:

### Compact TOML (used in this example)
```rust
to_toml(&node, &mut destination)
```
Produces TOML with minimal whitespace.

### Pretty-Printed TOML
```rust
to_toml_pretty(&node, &mut destination)
```
Produces formatted TOML with consistent spacing and table organization.

## Key Functions Used

- **`FileSource::new(path)`** - Opens a YAML file for reading
- **`parse(&mut source)`** - Parses YAML into a Node tree
- **`FileDestination::new(path)`** - Creates a destination file for writing
- **`to_toml(&node, &mut destination)`** - Converts Node tree to TOML format

## YAML to TOML Mapping

The conversion handles:
- **Mappings** → TOML tables
- **Nested mappings** → TOML nested tables `[table.subtable]`
- **Sequences** → TOML arrays
- **Strings** → TOML strings (with proper escaping)
- **Numbers** → TOML integers or floats
- **Booleans** → TOML true/false
- **Null** → Omitted (TOML doesn't have null)

Some YAML-specific features are adapted:
- **Anchors/aliases** - Resolved and expanded before conversion
- **Tags** - Applied during parsing, then converted
- **Multi-line strings** - Converted to TOML multi-line strings

## TOML Structure

TOML is more structured than YAML:
- **Tables** - Explicit section headers for nested data
- **Arrays of tables** - Special syntax for array of objects
- **No null** - Absent values are simply omitted
- **Inline tables** - Compact representation for small objects

Example TOML table structure:
```toml
# Root level key
name = "value"

# Table (like YAML mapping)
[database]
host = "localhost"
port = 5432

# Nested table
[database.connection_pool]
max_size = 10
timeout = 30

# Array of tables
[[servers]]
name = "alpha"
ip = "10.0.0.1"

[[servers]]
name = "beta"
ip = "10.0.0.2"
```

## Error Handling

The example demonstrates error handling for:
- Missing or unreadable files
- Invalid YAML syntax
- Incompatible YAML structures (e.g., complex keys)
- File write permissions
- Each file is processed independently

## TOML Limitations

When converting YAML to TOML, be aware:
- **No null values** - TOML doesn't support null
- **String keys only** - TOML table keys must be strings
- **No complex keys** - TOML doesn't support non-string keys
- **Limited nesting** - Very deep nesting can be verbose in TOML

## See Also

- **yaml_to_json** - Converting YAML to JSON format
- **yaml_to_xml** - Converting YAML to XML format
- **yaml_parse_and_stringify** - Basic YAML parsing and output
