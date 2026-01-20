//! Tests for anchor and alias parsing edge cases

use crate::Node;
use crate::test_helpers::parse_yaml;

#[cfg(test)]
mod tests {
    use super::*;

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
