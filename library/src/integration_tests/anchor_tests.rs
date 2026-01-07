//! Tests for anchor and alias parsing edge cases

use crate::Node;
use crate::io::sources::buffer::Buffer as BufferSource;
use crate::parse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_name_with_colon() {
        // Test from 2SXE: anchor name containing a colon
        let mut source = BufferSource::new(b"&a: key: &a value");
        let result = parse(&mut source);

        #[cfg(feature = "debug-trace")]
        println!("Result: {:?}", result);

        assert!(result.is_ok(), "Should parse anchor with colon in name");
        let doc = result.unwrap();

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
        let mut source = BufferSource::new(b"a: &anchor\nb: *anchor");
        let result = parse(&mut source);

        #[cfg(feature = "debug-trace")]
        println!("Result: {:?}", result);

        assert!(result.is_ok(), "Should parse anchor with empty/null value");
    }

    #[test]
    fn test_anchor_on_sequence() {
        // Test from 3R3P: anchor on a sequence
        let mut source = BufferSource::new(b"&sequence\n- a");
        let result = parse(&mut source);

        #[cfg(feature = "debug-trace")]
        println!("Result: {:?}", result);

        assert!(result.is_ok(), "Should parse anchor on sequence");
    }
}
