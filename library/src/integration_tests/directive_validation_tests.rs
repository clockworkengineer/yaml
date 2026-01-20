#[cfg(test)]
mod test_directive_validation {
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
