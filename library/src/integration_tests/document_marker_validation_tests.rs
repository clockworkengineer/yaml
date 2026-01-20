#[cfg(test)]
mod test_document_marker_validation {
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
