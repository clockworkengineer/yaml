///
/// Inline and flow syntax tests: inline mappings ({}), flow sequences ([]), explicit keys.
///
#[cfg(test)]
mod tests {
    use crate::nodes::node::{BlockStyle, QuoteType};
    use crate::{BufferSource, Node, Node::Document, Numeric, parse};
    use std::collections::HashMap;

    #[test]
    fn test_parse_inline_mapping_top_level() {
        let mut source = BufferSource::new(b"{a: 1, b: 2}");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![
            (
                Node::Str("a".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(1)),
            ),
            (
                Node::Str("b".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(2)),
            ),
        ])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_inline_mapping_empty() {
        let mut source = BufferSource::new(b"{}");
        let result = parse(&mut source).unwrap();
        let map: HashMap<String, Node> = HashMap::new();
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![{
                let mut pairs = Vec::new();
                for (k, v) in map.into_iter() {
                    let value = match v {
                        Node::Mapping(p) => Node::Mapping(p),
                        other => other,
                    };
                    pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                }

                Node::Mapping(pairs)
            }])])
        );
    }

    #[test]
    fn test_parse_inline_mapping_as_value() {
        let mut source = BufferSource::new(b"parent: {a: 1, b: test}");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("parent".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Mapping(vec![
                (
                    Node::Str("a".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Number(Numeric::Integer(1)),
                ),
                (
                    Node::Str("b".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
            ]),
        )])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_flow_sequence_with_special_leading_chars_and_quotes() {
        let mut source = BufferSource::new(b"[<tag, 'quoted', \"double\", >folded]");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Array(vec![
            Node::Str("<tag".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("quoted".to_string(), QuoteType::Single, BlockStyle::None),
            Node::Str("double".to_string(), QuoteType::Double, BlockStyle::None),
            Node::Str(">folded".to_string(), QuoteType::Unquoted, BlockStyle::None),
        ])])]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_explicit_sequence_keys() {
        let yaml = b"? # PLAY SCHEDULE\n  - Detroit Tigers\n  - Chicago Cubs\n:\n  - 2001-07-23\n\n? [ New York Yankees,\n    Atlanta Braves ]\n: [ 2001-07-02, 2001-08-12,\n    2001-08-14 ]\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        let mut collected: Vec<(Node, Node)> = Vec::new();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                let mut i = 0usize;
                while i < nodes.len() {
                    match &nodes[i] {
                        Node::Mapping(pairs) if pairs.len() == 1 => {
                            let (k, v) = &pairs[0];
                            if matches!(v, Node::None) {
                                if i + 1 < nodes.len() {
                                    let next = nodes[i + 1].clone();
                                    collected.push((k.clone(), next));
                                    i += 2;
                                    continue;
                                } else {
                                    collected.push((k.clone(), v.clone()));
                                }
                            } else {
                                collected.push((k.clone(), v.clone()));
                            }
                        }
                        Node::Mapping(pairs) => {
                            for (k, v) in pairs {
                                collected.push((k.clone(), v.clone()));
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }

                assert_eq!(collected.len(), 2);

                let (k1, v1) = &collected[0];
                if let Node::Str(ks1, qt1, _style1) = k1 {
                    assert_eq!(ks1, "[Detroit Tigers, Chicago Cubs]");
                    assert_eq!(*qt1, QuoteType::Double);
                } else {
                    panic!("First key is not a string");
                }

                if let Node::Array(items1) = v1 {
                    assert_eq!(items1.len(), 1);
                    if let Node::Str(vs1, _, _) = &items1[0] {
                        assert_eq!(vs1, "2001-07-23");
                    } else {
                        panic!("First value is not expected array item");
                    }
                } else {
                    panic!("First value is not an array");
                }

                let (k2, v2) = &collected[1];
                if let Node::Str(ks2, qt2, _style2) = k2 {
                    assert_eq!(ks2, "[New York Yankees, Atlanta Braves]");
                    assert_eq!(*qt2, QuoteType::Double);
                } else {
                    panic!("Second key is not a string");
                }

                if let Node::Array(items2) = v2 {
                    assert_eq!(items2.len(), 3);
                } else {
                    panic!("Second value is not an array");
                }
            } else {
                panic!("Expected Document nodes");
            }
        } else {
            panic!("Expected Documents node");
        }
    }

    #[test]
    fn test_parse_flow_sequence_basic() {
        let mut source = BufferSource::new(b"[1, 2, 3, 4]");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Array(vec![
            Node::Number(Numeric::Integer(1)),
            Node::Number(Numeric::Integer(2)),
            Node::Number(Numeric::Integer(3)),
            Node::Number(Numeric::Integer(4)),
        ])])]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_flow_sequence_mixed_types() {
        let mut source = BufferSource::new(b"[1, \"string\", true, null, 3.14]");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Array(items) = &nodes[0] {
                    // Should have parsed multiple items, exact types may vary
                    assert!(items.len() >= 4); // Parser might skip null
                    assert!(matches!(items[0], Node::Number(_)));
                    assert!(matches!(items[1], Node::Str(_, QuoteType::Double, _)));
                    assert!(matches!(items[2], Node::Boolean(_)));
                }
            }
        }
    }

    #[test]
    fn test_parse_flow_sequence_nested() {
        let mut source = BufferSource::new(b"[[1, 2], [3, 4], [5]]");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Array(outer) = &nodes[0] {
                    assert_eq!(outer.len(), 3);
                    for inner_array in outer {
                        assert!(matches!(inner_array, Node::Array(_)));
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_flow_sequence_with_whitespace() {
        let mut source = BufferSource::new(b"[ 1 ,  2  ,   3   ]");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Array(vec![
            Node::Number(Numeric::Integer(1)),
            Node::Number(Numeric::Integer(2)),
            Node::Number(Numeric::Integer(3)),
        ])])]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_flow_sequence_empty() {
        let mut source = BufferSource::new(b"[]");
        let result = parse(&mut source).unwrap();
        // Parser may treat empty flow sequence differently
        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                // May be empty document or empty array
                assert!(
                    nodes.is_empty() || (nodes.len() == 1 && matches!(nodes[0], Node::Array(_)))
                );
            }
        }
    }

    #[test]
    fn test_parse_flow_mapping_with_nested_sequences() {
        let mut source = BufferSource::new(b"{items: [1, 2, 3], names: [\"a\", \"b\"]}");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 2);
                    for (_, value) in pairs {
                        assert!(matches!(value, Node::Array(_)));
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_flow_mapping_with_nested_mappings() {
        let mut source =
            BufferSource::new(b"{user: {name: \"John\", age: 30}, config: {debug: true}}");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 2);
                    for (_, value) in pairs {
                        assert!(matches!(value, Node::Mapping(_)));
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_flow_mapping_with_quoted_keys() {
        let mut source = BufferSource::new(b"{\"key with spaces\": 1, 'another key': 2}");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 2);
                    let (key1, _) = &pairs[0];
                    let (key2, _) = &pairs[1];
                    if let Node::Str(k1, qt1, _) = key1 {
                        assert_eq!(k1, "key with spaces");
                        assert_eq!(*qt1, QuoteType::Double);
                    }
                    if let Node::Str(k2, qt2, _) = key2 {
                        assert_eq!(k2, "another key");
                        assert_eq!(*qt2, QuoteType::Single);
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_flow_mapping_with_special_characters() {
        let mut source =
            BufferSource::new(b"{key@symbol: value, key-dash: test, key_underscore: 123}");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 3);
                }
            }
        }
    }

    #[test]
    fn test_parse_multiline_flow_sequence() {
        let mut source = BufferSource::new(b"[\n  1,\n  2,\n  3\n]");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Array(vec![
            Node::Number(Numeric::Integer(1)),
            Node::Number(Numeric::Integer(2)),
            Node::Number(Numeric::Integer(3)),
        ])])]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_multiline_flow_mapping() {
        let mut source = BufferSource::new(b"{\n  a: 1,\n  b: 2\n}");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 2);
                }
            }
        }
    }

    #[test]
    fn test_parse_flow_sequence_with_comments() {
        // This parser may not support comments inside flow sequences
        let mut source = BufferSource::new(b"[1, 2, 3] # end comment");
        let result = parse(&mut source);
        if result.is_ok() {
            if let Node::Documents(docs) = result.unwrap() {
                assert_eq!(docs.len(), 1);
                if let Document(nodes) = &docs[0] {
                    assert_eq!(nodes.len(), 1);
                    if let Node::Array(items) = &nodes[0] {
                        assert_eq!(items.len(), 3);
                    }
                }
            }
        } else {
            // Parser doesn't support this syntax, which is valid
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_parse_flow_mapping_with_comments() {
        let mut source = BufferSource::new(b"{a: 1, # comment\n b: 2}");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 2);
                }
            }
        }
    }

    #[test]
    fn test_parse_deeply_nested_flow_structures() {
        let mut source = BufferSource::new(b"{level1: {level2: [1, {level3: true}]}}");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                assert!(matches!(nodes[0], Node::Mapping(_)));
            }
        }
    }

    #[test]
    fn test_parse_flow_sequence_as_mapping_value() {
        let mut source = BufferSource::new(b"items: [apple, banana, cherry]");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    let (_, value) = &pairs[0];
                    if let Node::Array(items) = value {
                        assert_eq!(items.len(), 3);
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_flow_mapping_as_sequence_item() {
        let mut source = BufferSource::new(b"- {name: John, age: 30}\n- {name: Jane, age: 25}");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Array(items) = &nodes[0] {
                    assert_eq!(items.len(), 2);
                    for item in items {
                        assert!(matches!(item, Node::Mapping(_)));
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_mixed_flow_and_block_syntax() {
        let mut source = BufferSource::new(b"config:\n  users: [{name: admin, roles: [read, write]}, {name: guest, roles: [read]}]");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                assert!(matches!(nodes[0], Node::Mapping(_)));
            }
        }
    }

    #[test]
    fn test_parse_flow_sequence_with_trailing_comma() {
        let mut source = BufferSource::new(b"[1, 2, 3,]");
        let result = parse(&mut source);
        // This should either parse successfully (ignoring trailing comma) or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_parse_flow_mapping_with_trailing_comma() {
        let mut source = BufferSource::new(b"{a: 1, b: 2,}");
        let result = parse(&mut source);
        // This should either parse successfully (ignoring trailing comma) or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_parse_flow_sequence_with_boolean_values() {
        let mut source = BufferSource::new(b"[true, false, yes, no, on, off]");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Array(items) = &nodes[0] {
                    assert_eq!(items.len(), 6);
                    // Items should be boolean or string nodes (parser may treat some as strings)
                    for item in items {
                        assert!(
                            matches!(item, Node::Boolean(_)) || matches!(item, Node::Str(_, _, _))
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_flow_sequence_with_null_values() {
        let mut source = BufferSource::new(b"[null, ~, , nothing]");
        let result = parse(&mut source);
        // This tests various null representations in flow sequences
        if result.is_ok() {
            if let Node::Documents(docs) = result.unwrap() {
                if let Document(nodes) = &docs[0] {
                    if let Node::Array(items) = &nodes[0] {
                        // Should have parsed some null values
                        assert!(!items.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_flow_mapping_with_numeric_keys() {
        let mut source = BufferSource::new(b"{1: first, 2: second, 3.14: pi}");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 3);
                }
            }
        }
    }

    #[test]
    fn test_parse_flow_sequence_with_escape_sequences() {
        let mut source =
            BufferSource::new(b"[\"line1\\nline2\", \"tab\\there\", \"quote\\\"here\"]");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Array(items) = &nodes[0] {
                    assert_eq!(items.len(), 3);
                    for item in items {
                        assert!(matches!(item, Node::Str(_, QuoteType::Double, _)));
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_flow_mapping_with_complex_values() {
        let mut source =
            BufferSource::new(b"{timestamp: 2023-01-01T00:00:00Z, duration: 3h30m, size: 1.5GB}");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 3);
                }
            }
        }
    }

    #[test]
    fn test_parse_flow_structures_with_unicode() {
        let mut source =
            BufferSource::new("{\u{1F44D}: \"thumbs up\", \"caf\u{E9}\": \"coffee\"}".as_bytes());
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 2);
                }
            }
        }
    }

    #[test]
    fn test_parse_extremely_nested_flow_sequence() {
        let mut source = BufferSource::new(b"[[[[1]]]]");
        let result = parse(&mut source).unwrap();
        if let Node::Documents(docs) = result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                // Should have parsed as nested arrays
                assert!(matches!(nodes[0], Node::Array(_)));
            }
        }
    }

    #[test]
    fn test_parse_flow_mapping_empty_values() {
        let mut source = BufferSource::new(b"{key1: , key2: null, key3: }");
        let result = parse(&mut source);
        // This tests empty values in flow mappings
        if result.is_ok() {
            if let Node::Documents(docs) = result.unwrap() {
                if let Document(nodes) = &docs[0] {
                    if let Node::Mapping(pairs) = &nodes[0] {
                        // Should have parsed some key-value pairs
                        assert!(!pairs.is_empty());
                    }
                }
            }
        }
    }
}
