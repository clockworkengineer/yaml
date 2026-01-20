//! Common test helpers for YAML library integration/unit tests

use crate::{Node, parse, BufferSource};

/// Parse YAML from a string or byte slice, panicking on error.
pub fn parse_yaml(input: impl AsRef<[u8]>) -> Node {
    let mut source = BufferSource::new(input.as_ref());
    parse(&mut source).expect("YAML parse failed")
}

/// Assert that two nodes are equal, with pretty diff on failure.
pub fn assert_nodes_eq(expected: &Node, actual: &Node) {
    assert_eq!(expected, actual, "Nodes are not equal.\nExpected: {:#?}\nActual: {:#?}", expected, actual);
}

/// Assert that parsing fails with a specific error substring.
pub fn assert_parse_error(input: impl AsRef<[u8]>, expected_msg: &str) {
    let mut source = BufferSource::new(input.as_ref());
    let result = parse(&mut source);
    assert!(result.is_err(), "Expected parse error, but got Ok");
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains(expected_msg), "Error message did not contain expected substring.\nExpected: {}\nActual: {}", expected_msg, err_str);
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
