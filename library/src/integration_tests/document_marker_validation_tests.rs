// =====================================================================================
//  File: document_marker_validation_tests.rs
//  Location: library/src/integration_tests/
// -------------------------------------------------------------------------------------
//  Purpose:
//      Integration tests for validation of YAML document marker usage in the yaml_lib crate.
//      These tests ensure that document start markers (---) are correctly validated for
//      placement, formatting, and error reporting according to the YAML specification.
//
//  Context:
//      - Part of the yaml_lib project, a Rust YAML parser/serializer.
//      - Focuses on error handling and validation for document marker placement and usage.
//      - Ensures robust error reporting and spec compliance for document marker issues.
//
// -------------------------------------------------------------------------------------
//  Test Coverage:
//      - Document start marker (---) placement and formatting
//      - Invalid content on marker line
//      - Proper marker usage with valid documents
//      - Error message validation for marker misuse
// =====================================================================================

#[cfg(test)]
mod tests {
    use crate::test_helpers::{assert_parse_error, parse_yaml};

    #[test]
    fn test_9kbc_mapping_on_document_start_line() {
        // Test: Content on same line as --- is invalid
        // YAML spec requires --- to be on its own line
        let yaml = b"--- key1: value1\n    key2: value2\n";
        assert_parse_error(yaml, "---");
    }

    #[test]
    fn test_valid_document_marker() {
        // Test: Correct usage with --- on separate line
        let yaml = b"---\nkey1: value1\nkey2: value2\n";
        let result = parse_yaml(yaml);
        assert!(
            matches!(result, crate::Node::Documents(_)),
            "Should parse valid document marker"
        );
    }
}
