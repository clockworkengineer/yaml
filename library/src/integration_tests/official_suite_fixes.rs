//! Tests for fixing the 10 failing official YAML test suite cases
//!
//! These tests recreate the failing test cases from the official YAML 1.2 test suite
//! to allow debugging and fixing without needing the full test suite data.

use crate::{BufferSource, Node, parse};

#[cfg(test)]
mod tests {
    use super::*;

    // Test 229Q - Spec Example 2.4. Sequence of Mappings
    #[test]
    fn test_229q_sequence_of_mappings() {
        let yaml = b"- name: Mark McGwire
  hr:   65
  avg:  0.278
- name: Sammy Sosa
  hr:   63
  avg:  0.288
";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        println!("229Q Result: {:?}", result);

        if let Ok(node) = result {
            // Should be a Document with an Array containing 2 Mappings
            if let Node::Document(docs) = node {
                if let Some(Node::Array(items)) = docs.first() {
                    assert_eq!(items.len(), 2, "Should have 2 items in sequence");
                    // Each item should be a mapping with 3 keys
                    for (i, item) in items.iter().enumerate() {
                        if let Node::Mapping(pairs) = item {
                            assert_eq!(pairs.len(), 3, "Item {} should have 3 key-value pairs", i);
                        } else {
                            panic!("Item {} should be a Mapping, got {:?}", i, item);
                        }
                    }
                } else {
                    panic!("Expected Array as first document element");
                }
            }
        } else {
            panic!("Failed to parse 229Q: {:?}", result.err());
        }
    }

    // Test 26DV - Whitespace around colon in mappings
    #[test]
    fn test_26dv_whitespace_around_colon() {
        // Testing various whitespace patterns around colons
        let yaml = b"key : value
key2  :  value2
key3:value3
";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        println!("26DV Result: {:?}", result);

        assert!(
            result.is_ok(),
            "Should parse whitespace around colons: {:?}",
            result.err()
        );
    }

    // Test 2CMS - Invalid mapping in plain multiline (false positive)
    #[test]
    fn test_2cms_plain_multiline() {
        // This might be a false positive - need to understand what makes it invalid
        let yaml = b"key: this is a plain
  multiline scalar
  that continues
";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        println!("2CMS Result: {:?}", result);

        // If this is marked as false positive, we should succeed
        assert!(
            result.is_ok(),
            "2CMS should parse (false positive): {:?}",
            result.err()
        );
    }

    // Test 36F6 - Multiline plain scalar with empty line
    #[test]
    fn test_36f6_multiline_with_empty_line() {
        let yaml = b"key: line one

  line two
";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        println!("36F6 Result: {:?}", result);

        // Empty lines in plain scalars are tricky
        // This test will help us understand the issue
        if result.is_err() {
            println!("36F6 Error: {:?}", result.err());
        }
    }

    // Test 3RLN - Leading tabs in double quoted strings
    #[test]
    fn test_3rln_tabs_in_double_quoted() {
        let yaml = b"key: \"\t\tvalue\"
";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        println!("3RLN Result: {:?}", result);

        assert!(
            result.is_ok(),
            "Should handle tabs in double-quoted strings: {:?}",
            result.err()
        );

        if let Ok(Node::Document(docs)) = result {
            if let Some(Node::Mapping(pairs)) = docs.first() {
                if let Some((_, Node::Str(value, _, _))) = pairs.first() {
                    assert!(value.contains('\t'), "Should preserve tab characters");
                }
            }
        }
    }

    // Test 4CQQ - Spec Example 2.18. Multi-line Flow Scalars
    #[test]
    fn test_4cqq_multiline_flow_scalars() {
        let yaml = b"plain:
  This unquoted scalar
  spans many lines.

quoted: \"So does this
  quoted scalar.\\n\"
";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        println!("4CQQ Result: {:?}", result);

        assert!(
            result.is_ok(),
            "Should parse multiline flow scalars: {:?}",
            result.err()
        );
    }

    // Test 4FJ6 - Nested implicit complex keys
    #[test]
    fn test_4fj6_nested_implicit_keys() {
        // Complex keys are keys that are themselves collections
        let yaml = b"? - key1
  - key2
: value
";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        println!("4FJ6 Result: {:?}", result);

        // This is an advanced feature - explicit complex key
        if result.is_err() {
            println!("4FJ6 Error: {:?}", result.err());
        }
    }

    // Test 4HVU - Wrong indentation in Sequence (false positive)
    #[test]
    fn test_4hvu_sequence_indentation() {
        let yaml = b"- item1
- item2
  - subitem1
  - subitem2
";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        println!("4HVU Result: {:?}", result);

        // If marked as false positive, we should succeed
        assert!(
            result.is_ok(),
            "4HVU should parse (false positive): {:?}",
            result.err()
        );
    }

    // Test 4ZYM - Spec Example 6.4. Line Prefixes
    #[test]
    fn test_4zym_line_prefixes() {
        let yaml = b"plain: text
  lines
folded: >
  text
  lines
literal: |
  text
  lines
";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        println!("4ZYM Result: {:?}", result);

        assert!(
            result.is_ok(),
            "Should handle line prefixes: {:?}",
            result.err()
        );
    }

    // Additional test for basic block scalar validation
    #[test]
    fn test_basic_sequence_of_mappings() {
        let yaml = b"- a: 1
  b: 2
- c: 3
  d: 4
";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        assert!(
            result.is_ok(),
            "Basic sequence of mappings should work: {:?}",
            result.err()
        );

        if let Ok(Node::Document(docs)) = result {
            if let Some(Node::Array(items)) = docs.first() {
                assert_eq!(items.len(), 2, "Should have 2 items");
            }
        }
    }
}
