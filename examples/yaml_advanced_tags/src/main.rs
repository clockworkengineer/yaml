//! Example demonstrating YAML advanced tags
//!
//! This example shows how to use YAML's advanced type tags:
//! - !!binary - Base64 encoded binary data
//! - !!omap - Ordered mappings
//! - !!pairs - Key-value pairs (allows duplicate keys)
//! - !!merge - Merge key for inheritance
//! - !!int:hex - Hexadecimal integers
//! - !!int:oct - Octal integers

use yaml_lib::{parse, stringify, BufferDestination, BufferSource};

fn main() {
    println!("=== YAML Advanced Tags Example ===\n");

    // Example 1: Binary data
    demo_binary_tag();

    // Example 2: Ordered mappings
    demo_omap_tag();

    // Example 3: Key-value pairs
    demo_pairs_tag();

    // Example 4: Merge keys
    demo_merge_tag();

    // Example 5: Numeric bases
    demo_numeric_bases();

    // Example 6: Combined example
    demo_combined_tags();
}

/// Demonstrates !!binary tag for base64-encoded data
fn demo_binary_tag() {
    println!("--- Example 1: Binary Data (!!binary) ---");

    let yaml = r#"
# Binary data encoded in base64
logo: !!binary |
  R0lGODlhDAAMAIQAAP//9/X17unp5WZmZgAAAOfn515eXvPz7Y6OjuDg4J+fn5
  OTk6enp56enmlpaWNjY6Ojo4SEhP/++f/++f/++f/++f/++f/++f/++f/++f/++Q==

# Inline binary data
favicon: !!binary "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="

# Empty binary data
empty_file: !!binary ""
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("Parsed binary data:");
            print_node(&node);
            println!("Note: Binary data is validated to be proper base64 format\n");
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Demonstrates !!omap tag for ordered mappings
fn demo_omap_tag() {
    println!("--- Example 2: Ordered Mappings (!!omap) ---");

    let yaml = r#"
# Regular mapping (order not guaranteed in all implementations)
regular_map:
  zebra: last
  apple: first
  mango: middle

# Ordered mapping (insertion order preserved)
ordered_map: !!omap
  - first: 1
  - second: 2
  - third: 3
  - fourth: 4

# Use case: form fields in specific order
form_fields: !!omap
  - username: { type: text, required: true }
  - email: { type: email, required: true }
  - password: { type: password, required: true }
  - confirm_password: { type: password, required: true }
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("Parsed ordered mappings:");
            print_node(&node);
            println!("Note: Order is preserved in !!omap\n");
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Demonstrates !!pairs tag for key-value pairs (allows duplicates)
fn demo_pairs_tag() {
    println!("--- Example 3: Key-Value Pairs (!!pairs) ---");

    let yaml = r#"
# Regular mapping doesn't allow duplicate keys
# The last value would override previous ones

# Pairs allow duplicate keys
http_headers: !!pairs
  - Content-Type: application/json
  - Accept: application/json
  - Accept: text/html
  - Set-Cookie: session=abc123
  - Set-Cookie: user=john_doe

# Use case: multiple values for same key
query_parameters: !!pairs
  - tag: rust
  - tag: yaml
  - tag: parsing
  - format: json
  - format: xml
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("Parsed pairs (duplicate keys allowed):");
            print_node(&node);
            println!("Note: !!pairs allows duplicate keys unlike regular mappings\n");
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Demonstrates !!merge tag for inheritance
fn demo_merge_tag() {
    println!("--- Example 4: Merge Keys (!!merge) ---");

    let yaml = r#"
# Base configuration
defaults: &DEFAULT
  timeout: 30
  retries: 3
  log_level: info

# Merge and override
development:
  !!merge <<: *DEFAULT
  log_level: debug
  debug_mode: true

# Multiple merges
production:
  timeout: 60
  retries: 5
  log_level: error
  monitoring: true

# Explicit merge tag (alternative syntax)
staging:
  !!merge <<: *DEFAULT
  log_level: warn
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("Parsed with merge keys:");
            print_node(&node);
            println!("Note: !!merge explicitly marks inheritance\n");
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Demonstrates hexadecimal and octal integers
fn demo_numeric_bases() {
    println!("--- Example 5: Numeric Bases ---");

    let yaml = r#"
# Hexadecimal integers
color_red: !!int:hex "FF0000"
color_green: !!int:hex "00FF00"
color_blue: !!int:hex "0000FF"
hex_value: !!int:hex "DEADBEEF"

# Octal integers
file_permissions: !!int:oct "755"
mask: !!int:oct "0644"
octal_value: !!int:oct "777"

# Regular integers for comparison
decimal: 255
hex_as_decimal: !!int:hex "FF"  # Same as 255
oct_as_decimal: !!int:oct "377" # Same as 255
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("Parsed numeric bases:");
            print_node(&node);
            println!("Note: Hex and octal are converted to integers\n");
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Demonstrates combining multiple advanced tags
fn demo_combined_tags() {
    println!("--- Example 6: Combined Advanced Tags ---");

    let yaml = r#"
# Application configuration using multiple tag types

application:
  name: MyApp
  version: 1.0.0
  
  # Binary assets
  assets: !!omap
    - logo: !!binary "R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7"
    - icon: !!binary "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII="
  
  # File permissions in octal
  permissions:
    config_file: !!int:oct "644"
    script_file: !!int:oct "755"
    log_dir: !!int:oct "755"
  
  # Color scheme in hex
  theme:
    primary_color: !!int:hex "007bff"
    secondary_color: !!int:hex "6c757d"
    success_color: !!int:hex "28a745"
    danger_color: !!int:hex "dc3545"
  
  # HTTP headers with duplicates
  default_headers: !!pairs
    - Content-Type: application/json
    - Accept: application/json
    - Accept: text/html
    - Cache-Control: no-cache
    - Cache-Control: no-store

# Configuration inheritance
base_config: &base
  timeout: 30
  retries: 3

environments: !!omap
  - development:
      !!merge <<: *base
      debug: true
  - staging:
      !!merge <<: *base
      log_level: info
  - production:
      !!merge <<: *base
      log_level: error
      timeout: 60
"#;

    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("Parsed combined advanced tags:");
            print_node(&node);
            println!("Note: Multiple tag types working together\n");
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

/// Helper function to print node structure
fn print_node(node: &yaml_lib::Node) {
    let mut dest = BufferDestination::new();
    match stringify(node, &mut dest) {
        Ok(_) => println!("{}", dest.to_string()),
        Err(e) => eprintln!("Stringify error: {}", e),
    }
}
