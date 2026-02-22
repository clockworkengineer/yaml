#[cfg(test)]
mod tests {
    use crate::test_helpers::{assert_parse_error, parse_yaml};

    #[test]
    fn test_9mag_leading_comma_in_sequence() {
        // Test: Leading comma in flow sequence (invalid)
        let yaml = b"---\n[ , a, b, c ]\n";
        assert_parse_error(yaml, "comma");
    }

    #[test]
    fn test_ctn5_double_comma_in_sequence() {
        // Test: Double comma in flow sequence (invalid)
        let yaml = b"---\n[ a, b, c, , ]\n";
        // Accept either "comma" or "consecutive" in error message
        let result = std::panic::catch_unwind(|| assert_parse_error(yaml, "comma"));
        if result.is_err() {
            assert_parse_error(yaml, "consecutive");
        }
    }

    #[test]
    fn test_cvw2_comment_without_space_after_comma() {
        // Test: Comment directly after comma with no space (invalid)
        // YAML spec requires whitespace before #
        let yaml = b"---\n[ a, b, c,#invalid\n]\n";
        assert_parse_error(yaml, "comment");
    }

    #[test]
    fn test_9jba_comment_without_space_after_bracket() {
        // Test: Comment directly after ] with no space (invalid)
        let yaml = b"---\n[ a, b, c, ]#invalid\n";
        assert_parse_error(yaml, "]");
    }

    // ...existing code...

    // ...existing code...

    #[test]
    fn test_5c5m_valid_trailing_comma() {
        // Test: Trailing comma (valid - should pass)
        let yaml = b"- { one : two , three: four , }\n- {five: six,seven : eight}";
        let result = parse_yaml(yaml);
        assert!(
            matches!(result, crate::Node::Documents(_)),
            "Should accept trailing comma in flow mapping"
        );
    }

    #[test]
    fn test_valid_sequence_trailing_comma() {
        // Test: Trailing comma in sequence (valid - should pass)
        let yaml = b"[1, 2, 3,]";
        let result = parse_yaml(yaml);
        assert!(
            matches!(result, crate::Node::Documents(_)),
            "Should accept trailing comma in flow sequence"
        );
    }
}
