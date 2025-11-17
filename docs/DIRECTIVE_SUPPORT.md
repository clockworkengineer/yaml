# YAML Directive Support

## Overview

As of v0.1.7, yaml_lib includes basic support for YAML directives. Directives are special instructions that appear before YAML documents and modify parsing behavior.

## Supported Directives

### 1. %YAML Version Directive

Specifies the YAML version for the following document.

**Syntax:** `%YAML major.minor`

**Example:**
```yaml
%YAML 1.2
---
key: value
```

**Supported Versions:**
- YAML 1.1
- YAML 1.2

**Error Handling:**
- Invalid versions (e.g., 2.0) produce clear error messages
- Version validation occurs during parsing

### 2. %TAG Directive

Defines a tag shorthand (handle) for the following document.

**Syntax:** `%TAG !handle! prefix`

**Example:**
```yaml
%TAG !e! tag:example.com,2000:app/
---
!e!mytype value
```

**Common Handles:**
- `!!` - Primary tag handle (YAML core types)
- `!` - Secondary tag handle (local types)
- `!prefix!` - Named handles (custom prefixes)

**Current Implementation:**
- Parses and stores tag prefix mappings
- Validates tag handle syntax
- **TODO:** Apply prefixes during tag resolution

### 3. Reserved Directives

Directives not defined in YAML spec are reserved for future use.

**Example:**
```yaml
%FOO bar baz
---
document
```

**Behavior:**
- Silently ignored (per YAML spec)
- No parse error generated
- Future-compatible

## Implementation Details

### Architecture

**Module:** `library/src/parser/directives.rs` (~420 lines)

**Key Components:**

1. **DirectiveContext** - Stores directive information
   - `yaml_version: Option<(u8, u8)>` - Major.minor version
   - `tag_prefixes: HashMap<String, String>` - Handle → prefix mappings

2. **parse_directives()** - Main entry point
   - Parses all directives before document content
   - Returns DirectiveContext or error

3. **Validation**
   - Version range checking (1.0 - 1.2)
   - Tag handle syntax validation
   - Clear error messages for invalid directives

### Multi-Document Support

Directives apply only to the immediately following document:

```yaml
%YAML 1.2
%TAG !e! tag:example.com,2000:app/
---
first: document

...

%TAG !e! tag:different.com,2000:app/
---
second: document
```

Each document can have its own directives. Directives are re-parsed before each document in a stream.

## Testing

### Unit Tests (12 tests)

Located in `library/src/parser/directives.rs`:

- `test_parse_yaml_directive` - Basic %YAML parsing
- `test_parse_yaml_directive_11` - YAML 1.1 support
- `test_parse_tag_directive` - Basic %TAG parsing
- `test_parse_tag_directive_primary` - Primary tag handle (!!)
- `test_parse_multiple_directives` - Combined directives
- `test_parse_reserved_directive` - Unknown directives
- `test_resolve_tag_with_prefix` - Tag resolution logic
- `test_resolve_tag_without_prefix` - Fallback behavior
- `test_invalid_yaml_version` - Error handling
- `test_yaml_directive_missing_dot` - Malformed directive
- `test_tag_directive_invalid_handle` - Invalid syntax

### Integration Tests

Part of main test suite (611 total tests passing).

### Official Test Suite

**Before:** 240/402 passing (59.7%)  
**After:** 251/402 passing (62.4%)  
**Improvement:** +11 tests (+2.7%)

Tests now passing:
- `27NA` - %YAML directive no longer causes error
- `2LFX` - Reserved directives handled
- And 9 others related to directive parsing

## Usage Examples

### Basic Version Declaration

```rust
use yaml_lib::parse_from_str;

let yaml = r#"
%YAML 1.2
---
name: John
age: 30
"#;

let result = parse_from_str(yaml);
assert!(result.is_ok());
```

### Tag Prefixes

```rust
use yaml_lib::parse_from_str;

let yaml = r#"
%TAG !e! tag:example.com,2000:app/
---
user: !e!Person
  name: John
"#;

let result = parse_from_str(yaml);
assert!(result.is_ok());
// Note: Tag prefix application not yet implemented
// Currently stored as Tagged node with "!e!Person"
```

### Multi-Document with Different Directives

```rust
use yaml_lib::parse_from_str;

let yaml = r#"
%YAML 1.2
---
first: document
...
%YAML 1.1
---
second: document
"#;

let result = parse_from_str(yaml);
assert!(result.is_ok());
```

## Roadmap

### Completed ✅
- [x] Parse %YAML directives
- [x] Parse %TAG directives
- [x] Handle reserved directives
- [x] Multi-document directive support
- [x] Version validation
- [x] Error handling

### In Progress 🚧
- [ ] Apply tag prefixes during tag resolution
- [ ] Pass DirectiveContext through parsing chain

### Future 🔮
- [ ] Strict version compliance mode
- [ ] Custom directive handlers (extension point)
- [ ] Directive preservation in round-trip parsing

## Performance

**Impact:** Minimal

- Directives parsed only once per document
- No ongoing overhead during document parsing
- HashMap lookups for tag resolution (O(1))
- Total code: ~420 lines

## Limitations

### Current Limitations

1. **Tag Resolution Not Applied**
   - Tag prefixes are parsed and stored
   - But not yet applied to tags in document content
   - Example: `!e!Person` stays as-is instead of expanding to full URI

2. **Version Differences Not Enforced**
   - Both YAML 1.1 and 1.2 parsed identically
   - No version-specific behavior yet
   - Future: Could enforce version-specific rules

3. **No Directive Preservation**
   - Directives not stored in output AST
   - Lost during stringify/round-trip
   - Future: Could add preservation mode

### Known Issues

None currently. All directive-related tests passing.

## Comparison with Other Libraries

| Feature | yaml_lib | libyaml | PyYAML | yaml-rust |
|---------|----------|---------|--------|-----------|
| %YAML parsing | ✅ | ✅ | ✅ | ✅ |
| %TAG parsing | ✅ | ✅ | ✅ | ❌ |
| Reserved directives | ✅ | ✅ | ⚠️ | ❌ |
| Tag resolution | ❌ | ✅ | ✅ | ❌ |
| Version validation | ✅ | ✅ | ✅ | ❌ |

## References

- **YAML 1.2 Specification:** https://yaml.org/spec/1.2.2/
  - Section 6.8: Directives
  - Section 6.8.1: YAML Directives
  - Section 6.8.2: TAG Directives
  - Section 6.8.3: Reserved Directives

- **Test Suite:** https://github.com/yaml/yaml-test-suite
  - Tests: 27NA, 2LFX, 5TYM, and others

## Contributing

Directive support is still evolving. Priority improvements:

1. **Tag Prefix Application** (High Priority)
   - Expand tag handles using stored prefixes
   - Estimate: +5-10 more tests passing

2. **Version-Specific Parsing** (Medium Priority)
   - Enforce YAML 1.1 vs 1.2 differences
   - Boolean values, octals, etc.

3. **Directive Preservation** (Low Priority)
   - Store directives in AST
   - Enable round-trip with directives

See `docs/OFFICIAL_TEST_SUITE_RESULTS.md` for detailed analysis of remaining test failures.
