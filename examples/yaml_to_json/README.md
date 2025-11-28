
# YAML to JSON Converter Example

This example demonstrates how to convert YAML files to JSON format using the YAML library's built-in JSON serialization capabilities. Error handling and validation are supported for robust conversion.


## What This Example Shows

- Reading YAML files using `FileSource`
- Parsing YAML content into Node structures
- Converting Node trees to JSON format with `to_json()`
- Writing JSON output to files using `FileDestination`
- Batch processing multiple files
- Error codes and recovery for malformed files

## How It Works

The example:
1. Scans the `files/` directory for all `.yaml` files
2. For each YAML file:
   - Opens and parses the YAML content
   - Converts the parsed Node tree to JSON format
   - Writes the JSON output to a new file with `.json` extension


This is useful for:
- **Data interchange** - Converting YAML configs to JSON for web APIs
- **Integration** - Working with systems that only accept JSON
- **Comparison** - Viewing YAML data in a different format
- **Web compatibility** - JSON is more widely supported in browsers
- **Error diagnostics** - Get detailed error codes and suggestions for invalid files

## Usage

```bash
# Run the example from the project root
cargo run --example yaml_to_json

# Or run from the examples directory
cd examples/yaml_to_json
cargo run
```

## Example Input/Output

### Input (`config.yaml`)
```yaml
server:
  host: localhost
  port: 8080
  ssl: true
database:
  name: mydb
  connections: 10
features:
  - authentication
  - caching
  - logging
```

### Output (`config.json`)
```json
{"server":{"host":"localhost","port":8080,"ssl":true},"database":{"name":"mydb","connections":10},"features":["authentication","caching","logging"]}
```

## JSON Formatting

The library provides two JSON output options:

### Compact JSON (used in this example)
```rust
to_json(&node, &mut destination)
```
Produces minified JSON without whitespace.

### Pretty-Printed JSON
```rust
to_json_pretty(&node, &mut destination)
```
Produces formatted JSON with indentation:
```json
{
  "server": {
    "host": "localhost",
    "port": 8080,
    "ssl": true
  }
}
```

## Key Functions Used

- **`FileSource::new(path)`** - Opens a YAML file for reading
- **`parse(&mut source)`** - Parses YAML into a Node tree
- **`FileDestination::new(path)`** - Creates a destination file for writing
- **`to_json(&node, &mut destination)`** - Converts Node tree to JSON format

## YAML Features in JSON

The conversion handles:
- **Mappings** → JSON objects
- **Sequences** → JSON arrays
- **Strings** → JSON strings
- **Numbers** → JSON numbers
- **Booleans** → JSON true/false
- **Null** → JSON null

Some YAML-specific features are adapted:
- **Anchors/aliases** - Resolved and expanded
- **Tags** - Applied during conversion
- **Multi-line strings** - Converted to single JSON strings

## Error Handling

The example demonstrates error handling for:
- Missing or unreadable files
- Invalid YAML syntax
- File write permissions
- Each file is processed independently

## See Also

- **yaml_to_xml** - Converting YAML to XML format
- **yaml_to_toml** - Converting YAML to TOML format
- **yaml_parse_and_stringify** - Basic YAML parsing and output
