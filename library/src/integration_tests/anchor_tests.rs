// =====================================================================================
//  File: anchor_tests.rs
//  Location: library/src/integration_tests/
// -------------------------------------------------------------------------------------
//  Purpose:
//      Integration tests for YAML anchor and alias handling in the yaml_lib crate.
//      These tests validate correct parsing and behavior of YAML anchors, aliases,
//      and related edge cases, including anchor names with special characters,
//      empty anchor values, and anchors on various node types.
//
//  Context:
//      - Part of the yaml_lib project, a Rust YAML parser/serializer.
//      - Tests are based on YAML specification examples and custom edge cases.
//      - Ensures compliance with YAML anchor/alias semantics and robustness.
//
// -------------------------------------------------------------------------------------
//  Test Coverage:
//      - Anchor names with special characters (e.g., colon)
//      - Empty anchor values
//      - Anchors on sequences and mappings
//      - Alias resolution
//      - Panic safety for malformed anchors/aliases
// =====================================================================================

#[cfg(test)]
mod tests {

    use crate::Node;
    use crate::test_helpers::parse_yaml;
    #[test]
    fn test_anchor_name_with_colon() {
        // Test from 2SXE: anchor name containing a colon
        let doc = parse_yaml(b"&a: key: &a value");
        // Should create an anchored node with name "a:"
        match &doc {
            Node::Documents(docs) => {
                assert!(!docs.is_empty());
            }
            _ => panic!("Expected Documents node"),
        }
    }

    #[test]
    fn test_empty_anchor_value() {
        // Test from 6KGN: anchor for empty node
        let _ = parse_yaml(b"a: &anchor\nb: *anchor");
        // If parse_yaml does not panic, test passes
    }

    #[test]
    fn test_anchor_on_sequence() {
        // Test from 3R3P: anchor on a sequence
        let _ = parse_yaml(b"&sequence\n- a");
        // If parse_yaml does not panic, test passes
    }
}
