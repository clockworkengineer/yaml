# YAML Advanced Tags Example

This example demonstrates YAML's advanced type tags that enable sophisticated data representation beyond basic scalars and collections. These tags provide type coercion, ordering guarantees, and specialized data handling.

## What This Example Shows

- **`!!binary`** - Base64-encoded binary data
- **`!!omap`** - Ordered mappings (insertion order preserved)
- **`!!pairs`** - Key-value pairs with duplicate keys allowed
- **`!!merge`** - Explicit merge key for inheritance
- **`!!int:hex`** - Hexadecimal integer notation
- **`!!int:oct`** - Octal integer notation

## Tag Descriptions

### 1. Binary Data (`!!binary`)

Encodes binary data as base64 strings:

```yaml
logo: !!binary |
  R0lGODlhDAAMAIQAAP//9/X17unp5WZmZgAAAOfn515eXvPz7Y6OjuDg4J+fn5
  OTk6enp56enmlpaWNjY6Ojo4SEhP/++f/++f/++f/++f/++f/++f/++f/++f/++Q==

favicon: !!binary "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
```

**Use cases:**
- Embedding images in configuration files
- Storing certificates or keys
- Including small binary assets
- Transmitting binary data in text format

**Features:**
- Automatically validates base64 format
- Supports multi-line literals
- Handles whitespace in base64 data
- Empty binary data is valid

### 2. Ordered Mappings (`!!omap`)

Preserves insertion order for mappings:

```yaml
form_fields: !!omap
  - username: { type: text, required: true }
  - email: { type: email, required: true }
  - password: { type: password, required: true }
  - confirm_password: { type: password, required: true }
```

**Use cases:**
- Form field ordering
- Step-by-step procedures
- Ranked lists with keys
- Priority-ordered configuration

**Important:**
- Guarantees order preservation
- Uses array-of-single-key-mappings syntax
- Each array item has exactly one key

### 3. Key-Value Pairs (`!!pairs`)

Allows duplicate keys in collections:

```yaml
http_headers: !!pairs
  - Content-Type: application/json
  - Accept: application/json
  - Accept: text/html
  - Set-Cookie: session=abc123
  - Set-Cookie: user=john_doe
```

**Use cases:**
- HTTP headers with multiple values
- Query parameters with repeated keys
- Event logs with timestamps
- Multi-value form data

**Features:**
- Duplicate keys are preserved
- Order is maintained
- Each pair is independent

### 4. Merge Keys (`!!merge`)

Explicit inheritance marker:

```yaml
defaults: &DEFAULT
  timeout: 30
  retries: 3

development:
  !!merge <<: *DEFAULT
  log_level: debug
```

**Use cases:**
- Configuration inheritance
- DRY (Don't Repeat Yourself) patterns
- Base/override scenarios
- Template expansion

### 5. Hexadecimal Integers (`!!int:hex`)

Integers in hexadecimal notation:

```yaml
color_red: !!int:hex "FF0000"
color_green: !!int:hex "00FF00"
memory_address: !!int:hex "DEADBEEF"
```

**Use cases:**
- Color codes
- Memory addresses
- Bit masks
- Hardware registers

### 6. Octal Integers (`!!int:oct`)

Integers in octal notation:

```yaml
file_permissions: !!int:oct "755"
umask: !!int:oct "0022"
mode: !!int:oct "644"
```

**Use cases:**
- Unix file permissions
- Legacy system compatibility
- Bit field specifications

## Usage

```bash
# Run the example
cargo run --example yaml_advanced_tags

# Or from the example directory
cd examples/yaml_advanced_tags
cargo run
```

## Output

The example demonstrates each tag type with:
1. Original YAML with the tag
2. Parsed representation
3. Stringified output
4. Explanatory notes

## Real-World Examples

### Configuration File with Mixed Tags

```yaml
application: !!omap
  - name: MyApp
  - version: 1.0.0
  - logo: !!binary "base64_encoded_logo_here"
  - permissions: !!int:oct "755"
  - theme_color: !!int:hex "007bff"

# Configuration inheritance
base_config: &base
  timeout: 30
  retries: 3

environments: !!omap
  - development:
      !!merge <<: *base
      debug: true
  - production:
      !!merge <<: *base
      timeout: 60
```

### HTTP Request Configuration

```yaml
request:
  method: POST
  url: https://api.example.com/data
  
  # Headers with duplicate keys
  headers: !!pairs
    - Content-Type: application/json
    - Accept: application/json
    - Accept: text/html
    - Authorization: Bearer token123
    
  # Binary payload
  body: !!binary "eyJkYXRhIjogInZhbHVlIn0="
```

### File System Configuration

```yaml
files: !!omap
  - config.yaml:
      path: /etc/app/config.yaml
      permissions: !!int:oct "644"
      owner: root
      
  - start.sh:
      path: /usr/local/bin/start.sh
      permissions: !!int:oct "755"
      owner: root
      
  - data.db:
      path: /var/lib/app/data.db
      permissions: !!int:oct "600"
      owner: app
```

## Tag Behavior

### Base64 Validation
```rust
// Valid base64:
"SGVsbG8gV29ybGQh"        // ✓
"SGVs bG8g V29y bGQh"      // ✓ (whitespace ignored)
"SGVsbG8gV29ybGQh=="      // ✓ (padding)

// Invalid base64:
"Hello World!"              // ✗ (not base64)
"SGVsbG8gV29ybGQ"          // ✗ (invalid length without padding)
```

### Numeric Base Conversion
```yaml
# All represent 255:
decimal: 255
hex: !!int:hex "FF"
octal: !!int:oct "377"

# All represent 0:
decimal_zero: 0
hex_zero: !!int:hex "0"
octal_zero: !!int:oct "0"
```

### Order Preservation
```yaml
# !!omap guarantees order
ordered: !!omap
  - third: 3
  - first: 1
  - second: 2

# Output maintains: third, first, second
```

## Limitations and Considerations

### Binary Data
- Large binary data inflates file size (base64 ~33% overhead)
- Consider external files for large binaries
- Limited to valid base64 characters

### Ordered Maps
- More verbose than regular mappings
- Array-of-mappings syntax required
- Each item must have exactly one key

### Pairs
- Not all YAML processors support duplicate keys
- Order-dependent processing
- Consider if regular arrays would suffice

### Numeric Bases
- Limited to integer types (no float hex/oct)
- String format required (quoted)
- Must be valid for the specified base

## Best Practices

1. **Use `!!binary` sparingly** - For small assets only
2. **`!!omap` when order matters** - Forms, steps, priorities
3. **`!!pairs` for multi-values** - HTTP headers, query params
4. **`!!merge` for clarity** - Makes inheritance explicit
5. **Hex for colors/masks** - More readable than decimal
6. **Octal for permissions** - Unix file permissions convention

## Error Handling

The example demonstrates handling:
- Invalid base64 data
- Malformed ordered maps
- Incorrect numeric formats
- Tag application errors
- Parse failures with descriptive messages

## Tag Compatibility

These tags are part of the YAML 1.2 specification:
- **Standard tags** - `!!binary`, `!!omap`, `!!pairs`, `!!merge`
- **Extended tags** - `!!int:hex`, `!!int:oct`
- **Widely supported** - Most YAML 1.2 parsers

## See Also

- **yaml_anchors_aliases** - Anchors and aliases for reuse
- **yaml_parse_and_stringify** - Basic YAML operations
- **YAML 1.2 Specification** - Official tag definitions
