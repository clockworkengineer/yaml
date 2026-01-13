//! Integration tests for validation and error detection

use crate::io::sources::buffer::Buffer;
use crate::parser::document::parse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_3hfz_invalid_content_after_document_end() {
        let yaml = b"---\nkey: value\n... invalid\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "Should reject content after document end marker"
        );
        if let Err(e) = result {
            assert!(
                e.contains("Invalid content") || e.contains("document end"),
                "Error: {}",
                e
            );
        }
    }

    #[test]
    fn test_4ejs_tabs_forbidden_as_indentation() {
        // Tabs as indentation in a mapping should be rejected
        let yaml = b"key1: value1\nkey2:\n\tvalue2"; // Tab before 'value2'
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "Should reject tabs as indentation: {:?}",
            result
        );
        if let Err(e) = result {
            assert!(
                e.to_lowercase().contains("tab"),
                "Error should mention tabs: {}",
                e
            );
        }
    }

    #[test]
    fn test_tabs_allowed_in_quoted_strings() {
        // Tabs inside quoted strings should be allowed
        let yaml = b"key: \"value\twith\ttab\"";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_ok(),
            "Tabs inside quoted strings should be allowed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_tabs_allowed_in_comments() {
        // Tabs in comments should be allowed
        let yaml = b"key: value  #\tcomment\twith\ttabs";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_ok(),
            "Tabs in comments should be allowed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_tabs_forbidden_in_flow_indentation() {
        // Tabs as indentation in flow collections should be rejected per YAML 1.2 spec
        let yaml = b"[\n\titem\n]";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "Should reject tabs as indentation in flow collections"
        );
        if let Err(e) = result {
            assert!(
                e.to_lowercase().contains("tab"),
                "Error should mention tabs: {}",
                e
            );
        }
    }

    #[test]
    #[ignore]
    fn test_4jvg_multiple_anchors_on_scalar() {
        let yaml = b"top1: &node1\n  &k1 key1: val1\ntop2: &node2\n  &v2 val2\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "Should reject multiple anchors on same node"
        );
    }

    #[test]
    #[ignore]
    fn test_7mnf_missing_colon() {
        let yaml = b"top1:\n  key1: val1\ntop2\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_err(), "Should reject mapping key without colon");
    }

    #[test]
    fn test_2g84_00_block_scalar_indent_zero() {
        let yaml = b"--- |0\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "Should reject block scalar with indentation indicator 0"
        );
        if let Err(e) = result {
            assert!(
                e.contains("indentation indicator") && e.contains("1-9"),
                "Error: {}",
                e
            );
        }
    }

    #[test]
    fn test_2g84_01_block_scalar_indent_ten() {
        let yaml = b"--- |10\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "Should reject block scalar with indentation indicator 10"
        );
        if let Err(e) = result {
            assert!(
                e.contains("single digit") && e.contains("1-9"),
                "Error: {}",
                e
            );
        }
    }

    #[test]
    fn test_g5u8_invalid_flow_dash_entries() {
        let yaml = b"---\n- [-, -]\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "Should reject flow sequence entries that are bare '-' indicators"
        );
    }
}
