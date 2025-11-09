# YAML Parse and Stringify Example

This example demonstrates the basic parsing and stringification functionality of the YAML library. It reads YAML files from a directory, parses them into a Node tree structure, and then stringifies them back to YAML format.

## What This Example Shows

- Reading YAML files using `FileSource`
- Parsing YAML content with the `parse()` function
- Writing YAML content using `FileDestination`
- Converting Node trees back to YAML with `stringify()`
- Batch processing multiple YAML files

## How It Works

The example:
1. Scans the `files/` directory for all `.yaml` files
2. For each file:
   - Opens it using `FileSource`
   - Parses the content into a `Node` tree structure
   - Creates a new output file with `.yaml.stringify` extension
   - Stringifies the Node tree back to YAML format
   - Writes the result to the output file

This is useful for:
- **Validating YAML syntax** - Files that parse successfully are valid YAML
- **Reformatting YAML** - The stringify process normalizes formatting
- **Testing round-trip conversion** - Verifying that parse→stringify preserves data
- **YAML normalization** - Converting various YAML styles to a canonical format

## Usage

```bash
# Run the example from the project root
cargo run --example yaml_parse_and_stringify

# Or run from the examples directory
cd examples/yaml_parse_and_stringify
cargo run
```

## Example Input/Output

### Input (`test.yaml`)
```yaml
name: John Doe
age: 30
skills:
  - Rust
  - YAML
  - Programming
address:
  city: San Francisco
  zip: 94105
```

### Output (`test.yaml.stringify`)
```yaml
name: John Doe
age: 30
skills:
  - Rust
  - YAML
  - Programming
address:
  city: San Francisco
  zip: 94105
```

The output maintains the data structure while applying consistent formatting.

## Key Functions Used

- **`FileSource::new(path)`** - Creates a source for reading from a file
- **`parse(&mut source)`** - Parses YAML into a Node tree
- **`FileDestination::new(path)`** - Creates a destination for writing to a file
- **`stringify(&node, &mut destination)`** - Converts Node tree to YAML format

## Error Handling

The example demonstrates proper error handling:
- File I/O errors are caught and reported
- Parse errors show which file failed
- Each file is processed independently (one failure doesn't stop others)

## See Also

- **yaml_to_json** - Converting YAML to JSON format
- **yaml_to_xml** - Converting YAML to XML format
- **yaml_fibonacci** - More advanced Node manipulation
