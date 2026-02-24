//! Test Helpers for YAML Library
//!
//! This module provides common helper functions for integration and unit tests in the YAML library.
//! It includes utilities for parsing YAML, asserting node equality, and simplifying test code.
//!
//! # Features
//! - Parse YAML from strings or bytes
//! - Assert node equality with pretty diffs
//! - Utilities for robust and readable tests
//!
//! # Usage
//! Use these helpers to write concise and effective tests for YAML processing code.

use crate::error::YamlError;
use crate::{BufferSource, Node, parse};

/// Parse YAML from a string or byte slice, panicking on error.
pub fn parse_yaml(input: impl AsRef<[u8]>) -> Node {
    // removed unused variable after switching to parse_with_config
    let config = crate::parser::config::ParserConfig::strict();
    crate::parse_with_config(
        std::str::from_utf8(input.as_ref()).expect("Invalid UTF-8"),
        config,
    )
    .expect("YAML parse failed")
}

/// Assert that two nodes are equal, with pretty diff on failure.
pub fn assert_nodes_eq(expected: &Node, actual: &Node) {
    assert_eq!(
        expected, actual,
        "Nodes are not equal.\nExpected: {:#?}\nActual: {:#?}",
        expected, actual
    );
}

/// Assert that parsing fails with a specific error substring.
pub fn assert_parse_error(input: impl AsRef<[u8]>, expected_msg: &str) {
    // removed unused variable after switching to parse_with_config
    let config = crate::parser::config::ParserConfig::strict();
    let result = crate::parse_with_config(
        std::str::from_utf8(input.as_ref()).expect("Invalid UTF-8"),
        config,
    );
    assert!(result.is_err(), "Expected parse error, but got Ok");
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains(expected_msg),
        "Error message did not contain expected substring.\nExpected: {}\nActual: {}",
        expected_msg,
        err_str
    );
}

/// Stringify a node to YAML (if supported by the build).
#[cfg(feature = "stringify")]
pub fn node_to_yaml_string(node: &Node) -> String {
    use crate::BufferDestination;
    use crate::stringify;
    let mut buf = BufferDestination::new();
    stringify(node, &mut buf).expect("Stringify failed");
    buf.to_string()
}

/// Perform a single round-trip (stringify -> parse) on a node.
///
/// This is intended for tests and property-based checks where we want to
/// verify that running through the full stringify/parse pipeline does not
/// crash and preserves structure.
#[cfg(feature = "stringify")]
pub fn roundtrip_node(node: &Node) -> Result<Node, YamlError> {
    use crate::BufferDestination;
    use crate::stringify;

    // Stringify the node to YAML
    let mut buf = BufferDestination::new();
    stringify(node, &mut buf)?;
    let yaml = buf.to_string();

    // Parse back into a Node
    let mut source = BufferSource::new(yaml.as_bytes());
    parse(&mut source)
}

/// Assert that a node is unchanged by a single stringify/parse round-trip.
#[cfg(feature = "stringify")]
pub fn assert_roundtrip_eq(node: &Node) {
    match roundtrip_node(node) {
        Ok(roundtripped) => {
            assert_nodes_eq(node, &roundtripped);
        }
        Err(err) => {
            panic!("Round-trip failed: {}", err);
        }
    }
}
