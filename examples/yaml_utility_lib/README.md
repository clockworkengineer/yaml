
# YAML Utility Library Example

This utility library provides helper functions for working with YAML files in the examples. It's designed to simplify common file operations across multiple examples and supports error handling and validation for robust workflows.


## What This Library Provides

- **File discovery** - Finding all YAML files in a directory
- **Batch processing** - Processing multiple files efficiently
- **Directory handling** - Creating directories as needed
- **Reusable patterns** - Common operations used across examples
- **Error codes and suggestions** - Enhanced error handling for file operations

## Main Function

### `get_yaml_file_list(file_path: &str) -> Vec<String>`

Scans a directory and returns a list of all `.yaml` file paths.

**Parameters:**
- `file_path` - Path to the directory to scan

**Returns:**
- `Vec<String>` - Vector of absolute paths to all `.yaml` files found

**Behavior:**
- Creates the directory if it doesn't exist
- Returns an empty vector if the directory is newly created
- Filters for files with `.yaml` extension only
- Returns full paths that can be used directly with `FileSource`

## Usage

### In Your Example

Add the utility library to your example's `Cargo.toml`:

```toml
[dependencies]
yaml_lib = { path = "../../library" }
yaml_utility_lib = { path = "../yaml_utility_lib" }
```

### In Your Code

```rust
use yaml_utility_lib::get_yaml_file_list;

fn main() {
    // Get all YAML files from the "files" directory
    let yaml_files = get_yaml_file_list("files");
    
    // Process each file
    for file_path in yaml_files {
        println!("Processing: {}", file_path);
        // Your processing logic here
    }
}
```

## Example Use Cases

### Batch File Conversion

```rust
use yaml_lib::{FileSource, parse, FileDestination, to_json};
use yaml_utility_lib::get_yaml_file_list;

fn main() {
    let yaml_files = get_yaml_file_list("files");
    
    for file_path in yaml_files {
        let mut source = FileSource::new(&file_path).unwrap();
        let node = parse(&mut source).unwrap();
        
        let json_path = file_path.replace(".yaml", ".json");
        let mut dest = FileDestination::new(&json_path).unwrap();
        to_json(&node, &mut dest).unwrap();
    }
}
```

### File Validation

```rust
use yaml_lib::{FileSource, parse};
use yaml_utility_lib::get_yaml_file_list;

fn main() {
    let yaml_files = get_yaml_file_list("files");
    let mut valid_count = 0;
    let mut invalid_count = 0;
    
    for file_path in yaml_files {
        let mut source = FileSource::new(&file_path).unwrap();
        match parse(&mut source) {
            Ok(_) => {
                println!("✓ Valid: {}", file_path);
                valid_count += 1;
            }
            Err(e) => {
                println!("✗ Invalid: {} - {}", file_path, e);
                invalid_count += 1;
            }
        }
    }
    
    println!("\nResults: {} valid, {} invalid", valid_count, invalid_count);
}
```

### Selective Processing

```rust
use yaml_utility_lib::get_yaml_file_list;

fn main() {
    let yaml_files = get_yaml_file_list("files");
    
    // Process only files matching a pattern
    for file_path in yaml_files {
        if file_path.contains("config") {
            println!("Processing config file: {}", file_path);
            // Process config files
        } else if file_path.contains("data") {
            println!("Processing data file: {}", file_path);
            // Process data files
        }
    }
}
```

## Implementation Details

The library uses standard Rust file system operations:
- `std::fs::read_dir()` for directory scanning
- `std::fs::create_dir()` for directory creation
- `std::path::Path` for path manipulation
- `filter_map()` for efficient file filtering

## Error Handling

The current implementation:
- Creates the directory if it doesn't exist (panics on failure)
- Panics if the directory can't be read (expects valid directory)
- Filters out entries that error during iteration
- Returns only valid `.yaml` file paths

For production use, you might want to:
- Return `Result` instead of panicking
- Provide more detailed error messages
- Handle permission errors gracefully
- Support recursive directory scanning

## Extending the Library

You could add functions for:

```rust
// Get files with any extension
pub fn get_file_list(dir: &str, extension: &str) -> Vec<String>

// Recursive directory scanning
pub fn get_yaml_files_recursive(dir: &str) -> Vec<String>

// File filtering with predicates
pub fn filter_yaml_files<F>(dir: &str, predicate: F) -> Vec<String>
where F: Fn(&str) -> bool

// Count files without loading paths
pub fn count_yaml_files(dir: &str) -> usize

// Get file metadata
pub fn get_yaml_file_info(dir: &str) -> Vec<(String, Metadata)>
```

## Used In Examples

This utility library is used by:
- **yaml_parse_and_stringify** - Batch YAML processing
- **yaml_to_json** - YAML to JSON conversion
- **yaml_to_xml** - YAML to XML conversion
- **yaml_to_toml** - YAML to TOML conversion
- **yaml_to_bencode** - YAML to Bencode conversion

## Directory Structure

The library expects (and creates if needed) this structure:

```
project_root/
├── files/           # Directory containing YAML files
│   ├── file1.yaml
│   ├── file2.yaml
│   └── file3.yaml
└── your_example/
    └── src/
        └── main.rs  # Your code using the utility
```

## Best Practices

When using this library:
1. **Create test files** - Put sample YAML files in `files/` directory
2. **Handle errors** - Wrap file operations in proper error handling
3. **Check results** - Verify the returned file list isn't empty
4. **Clean up** - Remove output files between test runs if needed

## See Also

- **yaml_parse_and_stringify** - Example using this library
- **yaml_to_json** - Another example using this library
- Rust `std::fs` module documentation
