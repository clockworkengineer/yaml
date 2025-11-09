# YAML to XML Converter Example

This example demonstrates how to convert YAML files to XML format using the YAML library's XML serialization capabilities.

## What This Example Shows

- Reading YAML files using `FileSource`
- Parsing YAML content into Node structures
- Converting Node trees to XML format with `to_xml()`
- Writing XML output to files using `FileDestination`
- Batch processing multiple files

## How It Works

The example:
1. Scans the `files/` directory for all `.yaml` files
2. For each YAML file:
   - Opens and parses the YAML content
   - Converts the parsed Node tree to XML format
   - Writes the XML output to a new file with `.xml` extension

This is useful for:
- **Legacy system integration** - Working with XML-based systems
- **Data transformation** - Converting modern YAML configs to XML
- **API compatibility** - Supporting XML-based APIs
- **Documentation generation** - XML formats for processing

## Usage

```bash
# Run the example from the project root
cargo run --example yaml_to_xml

# Or run from the examples directory
cd examples/yaml_to_xml
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

### Output (`config.xml`)
```xml
<root><server><host>localhost</host><port>8080</port><ssl>true</ssl></server><database><name>mydb</name><connections>10</connections></database><features><item>authentication</item><item>caching</item><item>logging</item></features></root>
```

## XML Formatting

The library provides two XML output options:

### Compact XML (used in this example)
```rust
to_xml(&node, &mut destination)
```
Produces minified XML without whitespace.

### Pretty-Printed XML
```rust
to_xml_pretty(&node, &mut destination)
```
Produces formatted XML with indentation:
```xml
<root>
  <server>
    <host>localhost</host>
    <port>8080</port>
    <ssl>true</ssl>
  </server>
  <database>
    <name>mydb</name>
    <connections>10</connections>
  </database>
  <features>
    <item>authentication</item>
    <item>caching</item>
    <item>logging</item>
  </features>
</root>
```

## Key Functions Used

- **`FileSource::new(path)`** - Opens a YAML file for reading
- **`parse(&mut source)`** - Parses YAML into a Node tree
- **`FileDestination::new(path)`** - Creates a destination file for writing
- **`to_xml(&node, &mut destination)`** - Converts Node tree to XML format

## YAML to XML Mapping

The conversion handles:
- **Mappings** → XML elements with child elements
- **Sequences** → XML elements with `<item>` children
- **Strings** → XML text nodes
- **Numbers** → XML text nodes (as strings)
- **Booleans** → XML text nodes ("true"/"false")
- **Null** → Empty XML elements

Some YAML-specific features are adapted:
- **Anchors/aliases** - Resolved and expanded before conversion
- **Tags** - Applied during parsing, then converted
- **Multi-line strings** - Preserved as XML text content

## Error Handling

The example demonstrates error handling for:
- Missing or unreadable files
- Invalid YAML syntax
- File write permissions
- Each file is processed independently

## XML Considerations

When converting YAML to XML, note:
- XML requires a single root element (automatically added if needed)
- Array items are wrapped in `<item>` tags by default
- Attribute-like data in YAML becomes child elements in XML
- XML special characters are automatically escaped

## See Also

- **yaml_to_json** - Converting YAML to JSON format
- **yaml_to_toml** - Converting YAML to TOML format
- **yaml_parse_and_stringify** - Basic YAML parsing and output
