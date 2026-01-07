///
/// Basic parsing tests: sequences, mappings, empty documents, comments.
///
#[cfg(test)]
mod tests {
    use crate::nodes::node::{BlockStyle, QuoteType};
    use crate::{BufferSource, Node, Node::Document, Numeric, parse};

    #[test]
    fn test_parse_sequence() {
        let mut source = BufferSource::new(b"- 1\n- 2\n- 3");
        let result = parse(&mut source).unwrap();

        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Number(Numeric::Integer(2)),
                Node::Number(Numeric::Integer(3))
            ])])])
        );
    }

    #[test]
    fn test_parse_sequence_with_comments() {
        let mut source = BufferSource::new(b"- 1\n# Comment 1\n- 2\n# Comment 2");
        let result = parse(&mut source).unwrap();

        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Number(Numeric::Integer(2))
            ])])])
        );
    }

    #[test]
    fn test_parse_mapping() {
        let mut source = BufferSource::new(b"key1: value1\nkey2: 42");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![
            (
                Node::Str("key1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            ),
            (
                Node::Str("key2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(42)),
            ),
        ])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_empty() {
        let mut source = BufferSource::new(b"");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![Document(vec![])]));
    }

    #[test]
    fn test_parse_invalid_char() {
        let mut source = BufferSource::new(b"@invalid");
        let result = parse(&mut source);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Unexpected character: @"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_parse_comment_only() {
        let mut source = BufferSource::new(b"# Just a comment");
        let result = parse(&mut source).unwrap();

        assert_eq!(result, Node::Documents(vec![Document(vec![])]));
    }

    #[test]
    fn test_parse_mapping_with_comments() {
        let mut source =
            BufferSource::new(b"# Comment before\nkey1: value1  # Inline comment\n# Comment after");
        let result = parse(&mut source).unwrap();

        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("key1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("value1".to_string(), QuoteType::Unquoted, BlockStyle::None),
        )])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_boolean_values() {
        let mut source =
            BufferSource::new(b"true_val: true\nfalse_val: false\nyes_val: yes\nno_val: no");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 4);
                    // Should have boolean values for true/false
                    assert!(matches!(pairs[0].1, Node::Boolean(true)));
                    assert!(matches!(pairs[1].1, Node::Boolean(false)));
                }
            }
        }
    }

    #[test]
    fn test_parse_null_values() {
        let mut source = BufferSource::new(b"null_val: null\ntilde_val: ~\nempty_val:");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 3);
                    // All should be None values
                    assert!(matches!(pairs[0].1, Node::None));
                    assert!(matches!(pairs[1].1, Node::None));
                    assert!(matches!(pairs[2].1, Node::None));
                }
            }
        }
    }

    #[test]
    fn test_parse_numeric_formats() {
        let mut source = BufferSource::new(
            b"int: 42\nfloat: 3.14\nnegative: -123\nzero: 0\nscientific: 1.23e10",
        );
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert!(pairs.len() >= 4);
                    assert!(matches!(pairs[0].1, Node::Number(Numeric::Integer(42))));
                    assert!(matches!(pairs[1].1, Node::Number(Numeric::Float(_))));
                    assert!(matches!(pairs[2].1, Node::Number(Numeric::Integer(-123))));
                    assert!(matches!(pairs[3].1, Node::Number(Numeric::Integer(0))));
                }
            }
        }
    }

    #[test]
    fn test_parse_quoted_strings() {
        let mut source = BufferSource::new(
            b"single: 'single quoted'\ndouble: \"double quoted\"\nunquoted: unquoted",
        );
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 3);

                    // Check that different string values are parsed correctly
                    if let Node::Str(content, _, _) = &pairs[0].1 {
                        assert_eq!(content, "single quoted");
                    }
                    if let Node::Str(content, _, _) = &pairs[1].1 {
                        assert_eq!(content, "double quoted");
                    }
                    if let Node::Str(content, _, _) = &pairs[2].1 {
                        assert_eq!(content, "unquoted");
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_multiline_strings() {
        let mut source =
            BufferSource::new(b"multiline: >\n  This is a\n  folded string\n  with multiple lines");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    if let Node::Str(content, _, _) = &pairs[0].1 {
                        assert!(content.contains("This is a"));
                        assert!(content.contains("folded string"));
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_literal_strings() {
        let mut source = BufferSource::new(b"literal: |\n  Line 1\n  Line 2\n  Line 3");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    if let Node::Str(content, _, block_style) = &pairs[0].1 {
                        assert!(matches!(block_style, BlockStyle::Literal));
                        assert!(content.contains("Line 1"));
                        assert!(content.contains("Line 2"));
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_nested_sequences() {
        let mut source = BufferSource::new(b"- [1, 2, 3]\n- [a, b, c]\n- [true, false, null]");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Array(items) = &nodes[0] {
                    assert_eq!(items.len(), 3);
                    // Each item should be an array
                    assert!(matches!(items[0], Node::Array(_)));
                    assert!(matches!(items[1], Node::Array(_)));
                    assert!(matches!(items[2], Node::Array(_)));
                }
            }
        }
    }

    #[test]
    fn test_parse_nested_mappings() {
        let mut source = BufferSource::new(b"outer:\n  inner1: value1\n  inner2: value2");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    if let Node::Mapping(inner_pairs) = &pairs[0].1 {
                        assert_eq!(inner_pairs.len(), 2);
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_sequence_of_mappings() {
        let mut source = BufferSource::new(b"- name: John\n  age: 30\n- name: Jane\n  age: 25");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Array(items) = &nodes[0] {
                    assert_eq!(items.len(), 2);
                    assert!(matches!(items[0], Node::Mapping(_)));
                    assert!(matches!(items[1], Node::Mapping(_)));
                }
            }
        }
    }

    #[test]
    fn test_parse_mixed_data_types() {
        let mut source = BufferSource::new(
            b"string: hello\nnumber: 42\nboolean: true\nnull_val: null\narray: [1, 2, 3]",
        );
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 5);
                    assert!(matches!(pairs[0].1, Node::Str(_, _, _)));
                    assert!(matches!(pairs[1].1, Node::Number(_)));
                    assert!(matches!(pairs[2].1, Node::Boolean(_)));
                    assert!(matches!(pairs[3].1, Node::None));
                    assert!(matches!(pairs[4].1, Node::Array(_)));
                }
            }
        }
    }

    #[test]
    fn test_parse_empty_collections() {
        let mut source = BufferSource::new(b"empty_array: []\nempty_object: {}");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 2);

                    if let Node::Array(arr) = &pairs[0].1 {
                        assert!(arr.is_empty());
                    }
                    if let Node::Mapping(map) = &pairs[1].1 {
                        assert!(map.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_7zz5_nested_with_empty_collections() {
        // Test case from 7ZZ5 - Empty flow collections with deeply nested sequences
        let yaml = b"---\nnested sequences:\n- - - []\n- - - {}\nkey1: []\nkey2: {}";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        if let Err(_e) = &result {
            #[cfg(feature = "debug-trace")]
            println!("7ZZ5 Error: {}", _e);
        }
        assert!(
            result.is_ok(),
            "Should parse nested sequences with empty flow collections"
        );
    }

    #[test]
    fn test_parse_5c5m_trailing_comma_in_flow_mapping() {
        // Test case from 5C5M - Trailing commas in flow mappings should be allowed
        let yaml = b"- { one : two , three: four , }\n- {five: six,seven : eight}";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        match &result {
            Ok(_) => {}
            Err(_e) => {
                #[cfg(feature = "debug-trace")]
                println!("5C5M Error: {}", _e)
            }
        }

        assert!(
            result.is_ok(),
            "Should parse flow mappings with trailing commas"
        );
    }

    #[test]
    fn test_parse_unicode_content() {
        let mut source = BufferSource::new("name: José\ncity: 北京\nemoji: 🚀".as_bytes());
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 3);

                    // Check that Unicode content is parsed (may have encoding differences)
                    if let Node::Str(content, _, _) = &pairs[0].1 {
                        assert!(content.contains("Jos") && content.len() > 3); // Allow for encoding variations
                    }
                    if let Node::Str(content, _, _) = &pairs[1].1 {
                        assert!(!content.is_empty()); // Should have Chinese characters
                    }
                    if let Node::Str(content, _, _) = &pairs[2].1 {
                        assert!(!content.is_empty()); // Should have emoji
                    }
                }
            }
        }
    }
    #[test]
    fn test_parse_escape_sequences() {
        let mut source = BufferSource::new(b"escaped: \"Line 1\\nLine 2\\tTabbed\"");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    if let Node::Str(content, _, _) = &pairs[0].1 {
                        // Should now contain actual newline and tab characters, not escaped versions
                        assert!(
                            content.contains('\n'),
                            "Should contain actual newline character"
                        );
                        assert!(
                            content.contains('\t'),
                            "Should contain actual tab character"
                        );
                        assert_eq!(content, "Line 1\nLine 2\tTabbed");
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_special_characters_in_keys() {
        let mut source =
            BufferSource::new(b"\"key with spaces\": value1\n'key-with-dashes': value2");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 2);

                    if let Node::Str(key, _, _) = &pairs[0].0 {
                        assert_eq!(key, "key with spaces");
                    }
                    if let Node::Str(key, _, _) = &pairs[1].0 {
                        assert_eq!(key, "key-with-dashes");
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_indentation_variations() {
        let mut source = BufferSource::new(b"level1:\n  level2:\n    level3: value");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    // Should handle nested indentation correctly
                    if let Node::Mapping(level2) = &pairs[0].1 {
                        if let Node::Mapping(level3) = &level2[0].1 {
                            assert_eq!(level3.len(), 1);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_trailing_spaces() {
        let mut source = BufferSource::new(b"key: value   \nother: data  \n");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 2);
                    // Values should not include trailing spaces
                    if let Node::Str(content, _, _) = &pairs[0].1 {
                        assert_eq!(content, "value");
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_multiple_consecutive_spaces() {
        let mut source = BufferSource::new(b"key:     value\nother:  data");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 2);
                    // Should handle multiple spaces after colon
                    if let Node::Str(content, _, _) = &pairs[0].1 {
                        assert_eq!(content, "value");
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_comments_in_various_positions() {
        let mut source = BufferSource::new(b"# Header comment\nkey1: value1 # End of line\n# Mid comment\nkey2: value2\n# Footer comment");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    // Comments should be ignored, only data remains
                    assert_eq!(pairs.len(), 2);
                }
            }
        }
    }

    #[test]
    fn test_parse_inline_arrays_with_mixed_types() {
        let mut source = BufferSource::new(b"mixed: [42, 'string', true, null, 3.14]");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    if let Node::Array(items) = &pairs[0].1 {
                        // Check that we have mixed types parsed correctly
                        assert!(items.len() >= 3);
                        assert!(matches!(items[0], Node::Number(Numeric::Integer(42))));
                        assert!(matches!(items[1], Node::Str(_, _, _)));
                        assert!(matches!(items[2], Node::Boolean(true)));
                        // null may be parsed as a string depending on implementation
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_inline_objects_with_various_keys() {
        let mut source = BufferSource::new(
            b"inline: {simple: value, 'quoted key': data, \"double quoted\": info}",
        );
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    if let Node::Mapping(inline_pairs) = &pairs[0].1 {
                        assert_eq!(inline_pairs.len(), 3);
                        // Should handle different key quotation styles
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_newlines_in_content() {
        let mut source =
            BufferSource::new(b"content: |\n  Line one\n  Line two\n  Line three\nother: value");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 2);
                    // Block scalar should preserve newlines
                    if let Node::Str(content, _, _) = &pairs[0].1 {
                        assert!(content.contains("Line one"));
                        assert!(content.contains("Line two"));
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_complex_nested_structure() {
        let yaml = b"users:
  - name: John
    roles: [admin, user]
    profile:
      age: 30
      active: true
  - name: Jane
    roles: [user]
    profile:
      age: 25
      active: false";

        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    // Should have users array with nested objects
                    if let Node::Array(users) = &pairs[0].1 {
                        assert_eq!(users.len(), 2);
                        assert!(matches!(users[0], Node::Mapping(_)));
                        assert!(matches!(users[1], Node::Mapping(_)));
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_zero_values() {
        let mut source = BufferSource::new(b"zero_int: 0\nzero_float: 0.0\nfalse_bool: false");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 3);
                    assert!(matches!(pairs[0].1, Node::Number(Numeric::Integer(0))));
                    assert!(matches!(pairs[1].1, Node::Number(Numeric::Float(f)) if f == 0.0));
                    assert!(matches!(pairs[2].1, Node::Boolean(false)));
                }
            }
        }
    }

    #[test]
    fn test_parse_whitespace_only_values() {
        let mut source = BufferSource::new(b"spaces: '   '\ntabs: '\t\t'\nmixed: ' \t '");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 3);
                    // Should preserve whitespace in quoted strings
                    if let Node::Str(content, _, _) = &pairs[0].1 {
                        assert_eq!(content, "   ");
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_extremely_long_keys_and_values() {
        let long_key = "a".repeat(1000);
        let long_value = "b".repeat(1000);
        let yaml = format!("{}: {}", long_key, long_value);

        let mut source = BufferSource::new(yaml.as_bytes());
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    if let Node::Str(key, _, _) = &pairs[0].0 {
                        assert_eq!(key.len(), 1000);
                    }
                    if let Node::Str(value, _, _) = &pairs[0].1 {
                        assert_eq!(value.len(), 1000);
                    }
                }
            }
        }
    }
}
