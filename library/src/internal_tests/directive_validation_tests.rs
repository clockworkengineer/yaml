// =====================================================================================
//  File: directive_validation_tests.rs
//  Location: library/src/internal_tests/
// -------------------------------------------------------------------------------------
//  Purpose:
//      Internal tests for validation of YAML directive usage in the yaml_lib crate.
//      These tests ensure that directives such as %YAML are correctly validated for
//      placement, required document content, and error reporting according to the YAML spec.
//
//  Context:
//      - Part of the yaml_lib project, a Rust YAML parser/serializer.
//      - Focuses on error handling and validation for directive placement and usage.
//      - Ensures robust error reporting and spec compliance for directive-related issues.
//
// -------------------------------------------------------------------------------------
//  Test Coverage:
//      - %YAML directive without document content
//      - Directives followed by end marker only
//      - Proper directive usage with valid documents
//      - Error message validation for directive misuse
// =====================================================================================

#[cfg(test)]
mod tests {
    use crate::test_helpers::{assert_parse_error, parse_yaml};

    #[test]
    fn test_9mma_directive_without_document() {
        // Test: %YAML directive with no document content
        let yaml = b"%YAML 1.2\n";
        assert_parse_error(yaml, "Directive must be followed by a document");
    }

    #[test]
    fn test_b63p_directive_with_only_end_marker() {
        // Test: Directive followed by ... with no document content
        let yaml = b"%YAML 1.2\n...\n";
        assert_parse_error(yaml, "Directive must be followed by a document");
    }

    #[test]
    fn test_valid_directive_with_document() {
        // Test: Proper directive followed by document content
        let yaml = b"%YAML 1.2\n---\nkey: value\n";
        let result = parse_yaml(yaml);
        assert!(
            matches!(result, crate::Node::Documents(_)),
            "Should parse valid directive with document"
        );
    }
}
