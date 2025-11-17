//! Integration tests for validation and error detection

use crate::io::sources::buffer::Buffer;
use crate::parser::document::parse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_3hfz_invalid_content_after_document_end() {
        let yaml = b"---\nkey: value\n... invalid\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_err(), "Should reject content after document end marker");
        if let Err(e) = result {
            assert!(e.contains("Invalid content") || e.contains("document end"), "Error: {}", e);
        }
    }

    // TODO: 4EJS - tabs as indentation
    // This is complex because tabs are only forbidden as INDENTATION, not in other contexts
    // Need more sophisticated checking that doesn't break valid YAML

    #[test]
    fn test_4jvg_multiple_anchors_on_scalar() {
        let yaml = b"top1: &node1\n  &k1 key1: val1\ntop2: &node2\n  &v2 val2\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_err(), "Should reject multiple anchors on same node");
    }

    #[test]
    fn test_7mnf_missing_colon() {
        let yaml = b"top1:\n  key1: val1\ntop2\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_err(), "Should reject mapping key without colon");
    }
}
