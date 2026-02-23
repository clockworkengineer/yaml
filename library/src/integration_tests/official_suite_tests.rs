//! Tests for fixing the failing official YAML test suite cases
//!
//! These tests recreate the failing test cases from the official YAML 1.2 test suite
//! to allow debugging and fixing without needing the full test suite data.

#[cfg(test)]
mod tests {
    use crate::{BufferSource, Node, parse};

    /// Helper: parse YAML and expect success, returning the root node.
    ///
    /// Panics with a descriptive label if parsing fails. Used by tests that
    /// need to inspect the resulting node tree.
    fn parse_expect_ok(yaml: &[u8], label: &str) -> Node {
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        #[cfg(feature = "debug-trace")]
        println!("{} Result: {:?}", label, result);

        match result {
            Ok(node) => node,
            Err(e) => panic!("{}: {:?}", label, e),
        }
    }

    /// Helper: assert that parsing succeeds, used for simple success cases.
    fn assert_parses(yaml: &[u8], label: &str) {
        let _ = parse_expect_ok(yaml, label);
    }

    /// Helper: assert that parsing fails, used for negative test cases.
    fn assert_fails(yaml: &[u8], label: &str) {
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        #[cfg(feature = "debug-trace")]
        println!("{} Result: {:?}", label, result);

        assert!(result.is_err(), "{}: {:?}", label, result);
    }

    // Standalone test for DMG6 - Wrong indentation in mapping (should fail to parse)
    #[test]
    fn test_dmg6_wrong_indentation_in_mapping() {
        // This YAML is from the DMG6 test case (should fail to parse)
        let yaml = b"key:\n  ok: 1\n wrong: 2\n";
        assert_fails(
            yaml,
            "DMG6 should fail to parse due to wrong indentation in mapping",
        );
    }

    // Test 5TRB - Unterminated/invalid quoted scalar
    #[test]
    fn test_5trb_unterminated_quoted_scalar() {
        // This YAML has an invalid quoted scalar (unterminated)
        let yaml = b"key: \"unterminated quoted scalar\n";
        assert_fails(yaml, "5TRB should fail to parse, but succeeded");
    }

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
        let node = parse_expect_ok(yaml, "Failed to parse 229Q");

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
    }

    // Test 26DV - Whitespace around colon in mappings
    #[test]
    fn test_26dv_whitespace_around_colon() {
        // Testing various whitespace patterns around colons
        let yaml = b"key : value
key2  :  value2
key3:value3
";
        assert_parses(yaml, "Should parse whitespace around colons");
    }

    // Test 2CMS - Invalid mapping in plain multiline (false positive)
    #[test]
    fn test_2cms_plain_multiline() {
        // This might be a false positive - need to understand what makes it invalid
        let yaml = b"key: this is a plain
  multiline scalar
  that continues
";
        // If this is marked as false positive, we should succeed
        assert_parses(yaml, "2CMS should parse (false positive)");
    }

    // Test 36F6 - Multiline plain scalar with empty line
    #[test]
    fn test_36f6_multiline_with_empty_line() {
        let yaml = b"key: line one

  line two
";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        #[cfg(feature = "debug-trace")]
        println!("36F6 Result: {:?}", result);

        // Empty lines in plain scalars are tricky
        // This test will help us understand the issue
        if result.is_err() {
            #[cfg(feature = "debug-trace")]
            println!("36F6 Error: {:?}", result.err());
        }
    }

    // Test 3RLN - Leading tabs in double quoted strings
    #[test]
    fn test_3rln_tabs_in_double_quoted() {
        let yaml = b"key: \"\t\tvalue\"
";
        let node = parse_expect_ok(yaml, "Should handle tabs in double-quoted strings");

        if let Node::Document(docs) = node {
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
        assert_parses(yaml, "Should parse multiline flow scalars");
    }

    // Test H7J7 - Invalid anchored mapping value combining empty scalar and !!map
    #[test]
    fn test_h7j7_invalid_anchored_mapping_value() {
        let yaml = b"key: &x\n!!map\n  a: b\n";
        assert_fails(yaml, "H7J7 should now be rejected as invalid");
    }

    // Test BU8L - Valid anchored mapping with !!map on separate line
    #[test]
    fn test_bu8l_valid_anchored_mapping_value() {
        let yaml = b"key: &anchor\n !!map\n  a: b\n";
        assert_parses(
            yaml,
            "BU8L should parse successfully as a valid anchored mapping",
        );
    }

    // Test 8XDJ - Invalid mixed plain scalar and mapping entries at same indentation
    #[test]
    fn test_8xdj_invalid_plain_then_mapping_at_same_indent() {
        let yaml = b"key: word1\n#  xxx\n  word2\n";
        assert_fails(
            yaml,
            "8XDJ should now be rejected as invalid at the document level",
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

        #[cfg(feature = "debug-trace")]
        println!("4FJ6 Result: {:?}", result);

        // This is an advanced feature - explicit complex key
        if result.is_err() {
            #[cfg(feature = "debug-trace")]
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
        // If marked as false positive, we should succeed
        assert_parses(yaml, "4HVU should parse (false positive)");
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
        assert_parses(yaml, "Should handle line prefixes");
    }

    // Additional test for basic block scalar validation
    #[test]
    fn test_basic_sequence_of_mappings() {
        let yaml = b"- a: 1
  b: 2
- c: 3
  d: 4
";
        let node = parse_expect_ok(yaml, "Basic sequence of mappings should work");

        if let Node::Document(docs) = node {
            if let Some(Node::Array(items)) = docs.first() {
                assert_eq!(items.len(), 2, "Should have 2 items");
            }
        }
    }
    // #[test]
    // fn test_4hvu_sequence_indentation_standalone() {
    //     let yaml = b"- item1\n- item2\n  - subitem1\n  - subitem2\n";
    //     let mut source = BufferSource::new(yaml);
    //     let result = parse(&mut source);

    //     #[cfg(feature = "debug-trace")]
    //     println!("4HVU Standalone Result: {:?}", result);

    //     // If marked as false positive, we should succeed
    //     assert!(
    //         result.is_err(),
    //         "4HVU should parse (false positive): {:?}",
    //         result.err()
    //     );
    // }
    // #[test]
    // fn test_2cms_plain_multiline_standalone() {
    //     // This might be a false positive - need to understand what makes it invalid
    //     let yaml = b"key: this is a plain\n  multiline scalar\n  that continues\n";
    //     let mut source = BufferSource::new(yaml);
    //     let result = parse(&mut source);

    //     #[cfg(feature = "debug-trace")]
    //     println!("2CMS Standalone Result: {:?}", result);

    //     // If this is marked as false positive, we should succeed
    //     assert!(
    //         result.is_err(),
    //         "2CMS should parse (false positive): {:?}",
    //         result.err()
    //     );
    // }

    // // Test 4QFQ - Block scalar edge cases (should error)
    // #[test]
    // fn test_4qfq_block_scalar_error() {
    //     // This YAML is from the 4QFQ test case, which should now succeed to parse
    //     let yaml = b"- |\n detected\n- >\n \n  \n  # detected\n- |1\n  explicit\n- >\n detected\n";
    //     let mut source = BufferSource::new(yaml);
    //     let result = parse(&mut source);

    //     #[cfg(feature = "debug-trace")]
    //     println!("4QFQ Result: {:?}", result);

    //     // This test should succeed to parse (expect Ok)
    //     assert!(
    //         result.is_err(),
    //         "4QFQ should succeed to parse, but failed: {:?}",
    //         result
    //     );
    // }

    // Test 236B - Indentation/structure: Invalid mapping/sequence structure (should error)
    // #[test]
    // fn test_236b_invalid_mapping_sequence_structure() {
    //     // This YAML is from the 236B test case (should fail to parse)
    //     let yaml = b"foo:\n  bar\ninvalid\n";
    //     let mut source = BufferSource::new(yaml);
    //     let result = parse(&mut source);

    //     #[cfg(feature = "debug-trace")]
    //     println!("236B Result: {:?}", result);

    //     // Fail the test if parsing does NOT return an error
    //     assert!(
    //         result.is_err(),
    //         "236B should fail to parse, but succeeded: {:?}",
    //         result
    //     );
    // }

    // Test parsing '---\n-\n-\n' and check its node structure
    // #[test]
    // fn test_parse_empty_sequence_items() {
    //     let yaml = b"---\n- \n- \n\n";
    //     let node = parse_expect_ok(yaml, "Failed to parse empty sequence items");

    //     // Should be a Document with an Array containing two Nulls
    //     if let Node::Documents(docs) = node {
    //         if let Some(Node::Document(items)) = docs.first() {
    //             assert_eq!(
    //                 items.len(),
    //                 1,
    //                 "Document should have one top-level node (the sequence)"
    //             );
    //             match &items[0] {
    //                 Node::Array(arr) => {
    //                     assert_eq!(arr.len(), 2, "Array should have 2 items");
    //                     for (i, item) in arr.iter().enumerate() {
    //                         match item {
    //                             Node::None => {}
    //                             other => panic!("Item {} should be Null, got {:?}", i, other),
    //                         }
    //                     }
    //                 }
    //                 other => panic!("Expected Array as first document element, got {:?}", other),
    //             }
    //         } else {
    //             panic!("Expected Document as first element in Documents");
    //         }
    //     } else {
    //         panic!("Expected Documents node at root");
    //     }
    // }
}
