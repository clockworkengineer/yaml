///
/// Integration tests for parsing and stringifying YAML content.
///
#[cfg(test)]
mod tests {
    use crate::nodes::node::{BlockStyle, QuoteType};
    use crate::{
        BufferDestination, BufferSource, FileSource, Node, Node::Document, Numeric, parse,
        stringify,
    };
    use std::collections::HashMap;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};


    fn normalize_newlines(s: &str) -> String {
        s.replace('\r', "")
    }

    fn get_json_file_paths(directory: &str) -> Vec<String> {
        let mut paths = Vec::new();
        if let Ok(entries) = fs::read_dir(directory) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                        if let Some(path_str) = path.to_str() {
                            paths.push(path_str.to_string());
                        }
                    }
                }
            }
        }
        paths
    }

    #[test]
    fn test_parse_yaml_files() {
        let files_dir = "../files";
        let json_files = get_json_file_paths(files_dir);
        for file_path in json_files {
            match FileSource::new(&file_path.to_string()) {
                Ok(mut source) => {
                    let result = parse(&mut source);
                    assert!(
                        result.is_ok(),
                        "Failed to parse {}: {:?}",
                        file_path,
                        result.err()
                    );
                }
                Err(e) => panic!("Failed to open {}: {}", file_path, e),
            }
        }
    }
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
    fn test_parse_multi_document() {
        let mut source = BufferSource::new(
            b"key1: value1\n---\nkey2: value2\n---\nkey3: value3\nkey4: value4\n",
        );
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![
            Document(vec![Node::Mapping(vec![(
                Node::Str("key1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            )])]),
            Document(vec![Node::Mapping(vec![(
                Node::Str("key2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value2".to_string(), QuoteType::Unquoted, BlockStyle::None),
            )])]),
            Document(vec![Node::Mapping(vec![
                (
                    Node::Str("key3".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("value3".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
                (
                    Node::Str("key4".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("value4".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
            ])]),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_header_comments() {
        let mut source = BufferSource::new(
            b"# Header comment 1\n# Header comment 2\n# Header comment 3\nkey: value\n",
        );
        let result = parse(&mut source).unwrap();

        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![{
                let mut map = HashMap::new();
                map.insert(
                    "key".to_string(),
                    Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
                );
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
    fn test_parse_nested_sequence() {
        let mut source = BufferSource::new(b"- item1\n- - nested1\n  - nested2\n- item2");
        let result = parse(&mut source).unwrap();
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![Node::Array(vec![
                Node::Str("item1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Array(vec![
                    Node::Str("nested1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("nested2".to_string(), QuoteType::Unquoted, BlockStyle::None)
                ]),
                Node::Str("item2".to_string(), QuoteType::Unquoted, BlockStyle::None)
            ])])])
        );
    }

    #[test]
    fn test_parse_mapping_with_comments() {
        let mut source = BufferSource::new(b"key1: value1\n# Comment 1\nkey2: 42\n# Comment 2");
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
    fn test_parse_nested_mapping() {
        let mut source = BufferSource::new(b"outer:\n  inner1: value1\n  inner2: value2");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("outer".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Mapping(vec![
                (
                    Node::Str("inner1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("value1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
                (
                    Node::Str("inner2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("value2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
            ]),
        )])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_nested_mapping_with_key_after_nested() {
        let mut source =
            BufferSource::new(b"outer1:\n  inner1: value1\n  inner2: value2\nouter2: value3");
        let result = parse(&mut source).unwrap();

        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![
            (
                Node::Str("outer1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Mapping(vec![
                    (
                        Node::Str("inner1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        Node::Str("value1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    ),
                    (
                        Node::Str("inner2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        Node::Str("value2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    ),
                ]),
            ),
            (
                Node::Str("outer2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value3".to_string(), QuoteType::Unquoted, BlockStyle::None),
            ),
        ])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_mapping_with_nested_sequence() {
        let mut source = BufferSource::new(b"key1:\n  - item1\n  - item2\nkey2: value2");
        let result = parse(&mut source).unwrap();

        let sequence = Node::Array(vec![
            Node::Str("item1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("item2".to_string(), QuoteType::Unquoted, BlockStyle::None),
        ]);

        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![
            (
                Node::Str("key1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                sequence,
            ),
            (
                Node::Str("key2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value2".to_string(), QuoteType::Unquoted, BlockStyle::None),
            ),
        ])])]);

        assert_eq!(result, expected);
    }
    #[test]
    fn test_parse_mapping_with_nested_sequence_and_comments() {
        let mut source = BufferSource::new(
            b"key1:\n  - item1\n  - item2\n# Comment 1\nkey2: value2\n# Comment 2",
        );
        let result = parse(&mut source).unwrap();
        let sequence = Node::Array(vec![
            Node::Str("item1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("item2".to_string(), QuoteType::Unquoted, BlockStyle::None),
        ]);
        let expected = Node::Documents(vec![Document(vec![

            Node::Mapping(vec![
                (
                    Node::Str("key1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    sequence,
                ),
                (
                    Node::Str("key2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("value2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
            ]),

        ])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_sequence_with_nested_comments() {
        let mut source = BufferSource::new(
            b"- item1\n# Comment between items\n- item2\n# Final comment\n- item3",
        );
        let result = parse(&mut source).unwrap();
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![Node::Array(vec![
                Node::Str("item1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("item2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("item3".to_string(), QuoteType::Unquoted, BlockStyle::None)
            ])])])
        );
    }

    #[test]
    fn test_parse_document_end_marker() {
        let mut source = BufferSource::new(b"key: value\n---");
        let result = parse(&mut source).unwrap();
        let mut expected = HashMap::new();
        expected.insert(
            "key".to_string(),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        );
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![{
                let mut pairs = Vec::new();
                for (k, v) in expected.into_iter() {
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
    fn test_parse_document_end_marker_with_trailing_content() {
        let mut source = BufferSource::new(b"key: value\n---\nother: 123");
        let result = parse(&mut source).unwrap();
        let mut doc1 = HashMap::new();
        doc1.insert(
            "key".to_string(),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        );
        let mut doc2 = HashMap::new();
        doc2.insert("other".to_string(), Node::Number(Numeric::Integer(123)));
        assert_eq!(
            result,
            Node::Documents(vec![
                Document(vec![{
                    let mut pairs = Vec::new();
                    for (k, v) in doc1.into_iter() {
                        let value = match v {
                            Node::Mapping(p) => Node::Mapping(p),
                            other => other,
                        };
                        pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                    }
                    Node::Mapping(pairs)
                }]),
                Document(vec![{
                    let mut pairs = Vec::new();
                    for (k, v) in doc2.into_iter() {
                        let value = match v {
                            Node::Mapping(p) => Node::Mapping(p),
                            other => other,
                        };
                        pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                    }
                    Node::Mapping(pairs)
                }])
            ])
        );
    }

    #[test]
    fn test_parse_document_end_marker_with_comments() {
        let mut source =
            BufferSource::new(b"# Comment before\nkey: value\n---\n# After doc\nother: 1");
        let result = parse(&mut source).unwrap();
        let mut doc1 = HashMap::new();
        doc1.insert(
            "key".to_string(),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        );
        let mut doc2 = HashMap::new();
        doc2.insert("other".to_string(), Node::Number(Numeric::Integer(1)));
        assert_eq!(
            result,
            Node::Documents(vec![
                Document(vec![{
                    let mut pairs = Vec::new();
                    for (k, v) in doc1.into_iter() {
                        let value = match v {
                            Node::Mapping(p) => Node::Mapping(p),
                            other => other,
                        };
                        pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                    }
                    Node::Mapping(pairs)
                }]),
                Document(vec![{
                    let mut pairs = Vec::new();
                    for (k, v) in doc2.into_iter() {
                        let value = match v {
                            Node::Mapping(p) => Node::Mapping(p),
                            other => other,
                        };
                        pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                    }
                    Node::Mapping(pairs)
                }])
            ])
        );
    }

    #[test]
    fn test_parse_document_end_marker_only() {
        let mut source = BufferSource::new(b"---");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![Document(vec![])]));
    }

    #[test]
    fn test_parse_multiple_document_end_markers() {
        let mut source = BufferSource::new(b"key: value\n---\n---\nother: 1");
        let result = parse(&mut source).unwrap();
        let mut doc1 = HashMap::new();
        doc1.insert(
            "key".to_string(),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        );
        let mut doc3 = HashMap::new();
        doc3.insert("other".to_string(), Node::Number(Numeric::Integer(1)));
        assert_eq!(
            result,
            Node::Documents(vec![
                Document(vec![{
                    let mut pairs = Vec::new();
                    for (k, v) in doc1.into_iter() {
                        let value = match v {
                            Node::Mapping(p) => Node::Mapping(p),
                            other => other,
                        };
                        pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                    }
                    Node::Mapping(pairs)
                }]),
                Document(vec![{
                    let mut pairs = Vec::new();
                    for (k, v) in doc3.into_iter() {
                        let value = match v {
                            Node::Mapping(p) => Node::Mapping(p),
                            other => other,
                        };
                        pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                    }

                    Node::Mapping(pairs)
                }])
            ])
        );
    }
    #[test]
    fn test_parse_nested_mapping_within_sequence() {
        let mut source = BufferSource::new(
            b"people:\n  - name: John\n    likes:\n      - apples\n      - bananas\n",
        );
        let result = parse(&mut source).unwrap();


        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("people".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Array(vec![Node::Mapping(vec![
                (
                    Node::Str("name".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("John".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
                (
                    Node::Str("likes".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Array(vec![
                        Node::Str("apples".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        Node::Str("bananas".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    ]),
                ),
            ])]),
        )])])]);

        assert_eq!(result, expected);
    }
    #[test]
    fn test_parse_sequence_of_mappings() {
        let yaml = b"-\n  name: Mark Joseph\n  hr: 87\n  avg: 0.278\n-\n  name: James Stephen\n  hr: 63\n  avg: 0.288\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();


        let mut mark_map = HashMap::new();
        mark_map.insert(
            "name".to_string(),
            Node::Str(
                "Mark Joseph".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ),
        );
        mark_map.insert("hr".to_string(), Node::Number(Numeric::Integer(87)));
        mark_map.insert("avg".to_string(), Node::Number(Numeric::Float(0.278)));

        let mut james_map = HashMap::new();
        james_map.insert(
            "name".to_string(),
            Node::Str(
                "James Stephen".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ),
        );
        james_map.insert("hr".to_string(), Node::Number(Numeric::Integer(63)));
        james_map.insert("avg".to_string(), Node::Number(Numeric::Float(0.288)));

        let expected = Node::Documents(vec![Document(vec![Node::Array(vec![
            Node::Mapping(vec![
                (
                    Node::Str("name".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str(
                        "Mark Joseph".to_string(),
                        QuoteType::Unquoted,
                        BlockStyle::None,
                    ),
                ),
                (
                    Node::Str("hr".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Number(Numeric::Integer(87)),
                ),
                (
                    Node::Str("avg".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Number(Numeric::Float(0.278)),
                ),
            ]),
            Node::Mapping(vec![
                (
                    Node::Str("name".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str(
                        "James Stephen".to_string(),
                        QuoteType::Unquoted,
                        BlockStyle::None,
                    ),
                ),
                (
                    Node::Str("hr".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Number(Numeric::Integer(63)),
                ),
                (
                    Node::Str("avg".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Number(Numeric::Float(0.288)),
                ),
            ]),
        ])])]);

        assert_eq!(result, expected);
    }

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
    fn test_block_scalar_like_string_same_line() {

        let mut source = BufferSource::new(b"key: > hello world");
        let result = parse(&mut source).unwrap();
        let mut expected = HashMap::new();
        expected.insert(
            "key".to_string(),
            Node::Str(
                "> hello world".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ),
        );
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![{
                let mut pairs = Vec::new();
                for (k, v) in expected.into_iter() {
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
    fn test_block_scalar_like_string_next_line() {

        let mut source = BufferSource::new(b"key:\n  > multi line");
        let result = parse(&mut source).unwrap();
        let mut expected = HashMap::new();
        expected.insert(
            "key".to_string(),
            Node::Str(
                "> multi line".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ),
        );
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![{
                let mut pairs = Vec::new();
                for (k, v) in expected.into_iter() {
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
    fn test_parse_and_preserve_local_tag_on_scalar() {


        let mut source = BufferSource::new(b"value: !!str 123");
        let result = parse(&mut source).unwrap();


        if let Node::Documents(ref docs) = result {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    let (_k, v) = &pairs[0];
                    if let Node::Str(s, qt, _style) = v {
                        println!("FOUND_STR s='{}' qt={:?}", s, qt);
                        assert_eq!(s.as_str(), "123");
                        assert_eq!(*qt, QuoteType::Unquoted);
                        return;
                    }
                }
            }
        }

        println!("PARSED_RESULT: {:#?}", result);
        panic!("Expected coerced string node not found");
    }

    #[test]
    fn test_stringify_preserves_tag_token() {


        let mut source = BufferSource::new(b"value: !!str 123");
        let node = parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();

        if !out.contains("\"123\"") {
            println!("STRINGIFIED_OUT:\n{}", out);
        }
        assert!(
            out.contains("value: 123"),
            "stringify should include the coerced value: {}",
            out
        );
    }

    #[test]
    fn test_coerce_int_tag_on_numeric_string() {

        let mut source = BufferSource::new(b"value: !!int '123'");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(ref docs) = result {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    let (_k, v) = &pairs[0];
                    assert_eq!(v, &Node::Number(Numeric::Integer(123)));
                    return;
                }
            }
        }
        panic!("Expected coerced integer node not found");
    }

    #[test]
    fn test_coerce_float_tag_on_numeric_string() {

        let mut source = BufferSource::new(b"value: !!float '3.14'");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(ref docs) = result {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    let (_k, v) = &pairs[0];
                    assert_eq!(v, &Node::Number(Numeric::Float(3.14)));
                    return;
                }
            }
        }
        panic!("Expected coerced float node not found");
    }

    #[test]
    fn test_coerce_float_tag_on_integer_value() {

        let mut source = BufferSource::new(b"value: !!float 2");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(ref docs) = result {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    let (_k, v) = &pairs[0];
                    assert_eq!(v, &Node::Number(Numeric::Float(2.0)));
                    return;
                }
            }
        }
        panic!("Expected coerced float from integer not found");
    }

    #[test]
    fn test_coerce_float_tag_on_negative_float_value() {

        let mut source = BufferSource::new(b"value: !!float -2.0");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(ref docs) = result {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    let (_k, v) = &pairs[0];
                    assert_eq!(v, &Node::Number(Numeric::Float(-2.0)));
                    return;
                }
            }
        }
        panic!("Expected coerced float from integer not found");
    }

    #[test]
    fn test_coerce_float_tag_on_negative_numeric_string() {

        let mut source = BufferSource::new(b"value: !!float '-2.5'");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(ref docs) = result {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    let (_k, v) = &pairs[0];
                    assert_eq!(v, &Node::Number(Numeric::Float(-2.5)));
                    return;
                }
            }
        }
        panic!("Expected coerced negative float node not found");
    }

    #[test]
    fn test_coerce_int_tag_on_negative_numeric_string() {

        let mut source = BufferSource::new(b"value: !!int '-2'");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(ref docs) = result {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    let (_k, v) = &pairs[0];
                    assert_eq!(v, &Node::Number(Numeric::Integer(-2)));
                    return;
                }
            }
        }
        panic!("Expected coerced negative integer node not found");
    }

    #[test]
    fn test_coerce_float_tag_on_negative_integer_value() {

        let mut source = BufferSource::new(b"value: !!float -2");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(ref docs) = result {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    let (_k, v) = &pairs[0];
                    assert_eq!(v, &Node::Number(Numeric::Float(-2.0)));
                    return;
                }
            }
        }
        panic!("Expected coerced float from negative integer not found");
    }

    #[test]
    fn test_coerce_timestamp_on_date_string() {

        let mut source = BufferSource::new(b"value: !!timestamp '2001-12-14'");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(ref docs) = result {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    let (_k, v) = &pairs[0];
                    if let Node::Str(s, _, _) = v {
                        assert_eq!(s, "2001-12-14");
                        return;
                    }
                }
            }
        }
        panic!("Expected coerced timestamp string node not found");
    }

    #[test]
    fn test_coerce_timestamp_on_rfc3339_datetime() {

        let mut source = BufferSource::new(b"value: !!timestamp 2001-12-14T21:59:43Z");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(ref docs) = result {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    let (_k, v) = &pairs[0];
                    if let Node::Str(s, _, _) = v {
                        assert_eq!(s, "2001-12-14T21:59:43Z");
                        return;
                    }
                }
            }
        }
        panic!("Expected coerced RFC3339 timestamp string node not found");
    }

    #[test]
    fn test_unknown_tag_is_preserved_and_stringified() {

        let mut source = BufferSource::new(b"value: !custom foo");
        let node = parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        assert!(
            out.contains("!custom foo"),
            "stringify should include unknown tag: {}",
            out
        );
    }

    #[test]
    fn test_tag_on_sequence_is_preserved() {

        let mut source = BufferSource::new(b"value: !seq - 1\n  - 2");
        let node = parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        assert!(
            out.contains("!seq"),
            "stringify should preserve sequence tag: {}",
            out
        );
    }

    #[test]
    fn test_tagged_anchor_and_alias_resolution() {

        let yaml = b"---\na: &a !!str 123\nb: *a\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {

                    let mut found_a = None;
                    let mut found_b = None;
                    for (k, v) in pairs {
                        if let Node::Str(ks, _, _) = k {
                            if ks == "a" {
                                found_a = Some(v.clone());
                            }
                            if ks == "b" {
                                found_b = Some(v.clone());
                            }
                        }
                    }
                    assert!(found_a.is_some() && found_b.is_some());
                    assert_eq!(found_a.unwrap(), found_b.unwrap());
                    return;
                }
            }
        }
        panic!("Tagged anchor/alias resolution failed");
    }

    #[test]
    fn test_flow_multiline_double_quoted_in_sequence() {
        let yaml = b"[\n\"line1\nline2\", 2\n]";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Array(vec![
            Node::Str(
                "line1\nline2".to_string(),
                QuoteType::Double,
                BlockStyle::None,
            ),
            Node::Number(Numeric::Integer(2)),
        ])])]);
        assert_eq!(result, expected);
    }
    #[test]
    fn test_flow_multiline_single_quoted_mapping_value() {
        let yaml = b"{a: 'hello\nworld'}";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("a".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str(
                "hello\nworld".to_string(),
                QuoteType::Single,
                BlockStyle::None,
            ),
        )])])]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_flow_multiline_quoted_key_in_inline_mapping() {
        let yaml = b"{\"multi\nline\": 1}";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str(
                "multi\nline".to_string(),
                QuoteType::Double,
                BlockStyle::None,
            ),
            Node::Number(Numeric::Integer(1)),
        )])])]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_empty_document_end_marker() {
        let mut source = BufferSource::new(b"...");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![Document(vec![])]));
    }

    #[test]
    fn test_parse_literal_block_literal_scalar_with_indent() {
        let yaml = b"---\nstring1: |\n  Line1\n  line2\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("string1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str(
                "  Line1\n  line2".to_string(),
                QuoteType::Unquoted,
                BlockStyle::Literal,
            ),
        )])])]);
        assert_eq!(result, expected);
    }
    #[test]
    fn test_parse_literal_block_folded_scalar_with_indent() {
        let yaml = b"---\nstring1: >\n  Line1\n  line2\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_mapping_with_inline_comment_and_indented_sequence() {
        let yaml = b"---\nhr: # 1998 hr ranking\n  - Mark McGwire\n  - Sammy Sosa\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("hr".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Array(vec![
                Node::Str(
                    "Mark McGwire".to_string(),
                    QuoteType::Unquoted,
                    BlockStyle::None,
                ),
                Node::Str(
                    "Sammy Sosa".to_string(),
                    QuoteType::Unquoted,
                    BlockStyle::None,
                ),
            ]),
        )])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_anchor_and_alias_in_mapping() {
        let yaml = b"---\nanchor: &a \n  nested: value\nalias_ref: *a\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();


        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_anchor_and_alias_in_sequence() {

        let yaml = b"---\n- &a hello\n- *a\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();


        if let Node::Documents(docs) = result {
            if let Document(nodes) = &docs[0] {
                if let Node::Array(items) = &nodes[0] {
                    assert_eq!(
                        items[0],
                        Node::Str("hello".to_string(), QuoteType::Unquoted, BlockStyle::None)
                    );
                    assert_eq!(
                        items[1],
                        Node::Str("hello".to_string(), QuoteType::Unquoted, BlockStyle::None)
                    );
                    return;
                }
            }
        }
        panic!("Unexpected parse result structure");
    }
    #[test]
    fn test_parse_nested_anchor_and_alias() {

        let yaml = b"---\nroot: &a\n  nested:\n    value: 1\nref: *a\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();


        println!("TEST_PARSED: {:#?}", result);


        if let Node::Documents(docs) = result {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {

                    let mut found = false;
                    for (k, v) in pairs {
                        if let Node::Str(ks, _, _) = k {
                            if ks == "ref" {

                                if let Node::Mapping(inner_pairs) = v {

                                    let mut ok = false;
                                    for (ik, _iv) in inner_pairs {
                                        if let Node::Str(iks, _, _) = ik {
                                            if iks == "nested" {
                                                ok = true;
                                            }
                                        }
                                    }
                                    assert!(ok);
                                    found = true;
                                }
                            }
                        }
                    }
                    assert!(found);
                    return;
                }
            }
        }
        panic!("Unexpected parse result structure");
    }

    #[test]
    fn test_parse_undefined_alias_errors() {
        let mut source = BufferSource::new(b"---\nvalue: *nope\n");
        let res = parse(&mut source);
        assert!(res.is_err());
    }

    #[test]
    fn test_error_on_empty_alias_name() {
        use crate::error::messages::ERR_EMPTY_ALIAS_NAME;
        let mut source = BufferSource::new(b"---\nvalue: *\n");
        let res = parse(&mut source);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains(ERR_EMPTY_ALIAS_NAME));
    }

    #[test]
    fn test_error_on_empty_anchor_name() {
        use crate::error::messages::ERR_EMPTY_ANCHOR_NAME;
        let mut source = BufferSource::new(b"---\nroot: &\n  nested: 1\n");
        let res = parse(&mut source);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains(ERR_EMPTY_ANCHOR_NAME));
    }

    #[test]
    fn test_error_on_duplicate_anchor() {
        use crate::error::messages::ERR_DUPLICATE_ANCHOR_PREFIX;
        let yaml = b"---\na: &dup\n  x: 1\nb: &dup\n  y: 2\n";
        let mut source = BufferSource::new(yaml);
        let res = parse(&mut source);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains(ERR_DUPLICATE_ANCHOR_PREFIX));
    }

    #[test]
    fn test_parse_merge_key_with_single_alias() {
        let yaml = b"---\na: &a\n  nested: anchor\nparent:\n  <<: *a\n  key: value\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();
        assert!(matches!(result, Node::Documents(_)));
    }
    #[test]
    fn test_parse_anchor_alias_sequence_hr_rbi() {

        let yaml = b"---\nhr:\n  - Mark McGwire\n  # Following node labeled SS\n  - &SS Sammy Sosa\nrbi:\n  - *SS # Subsequent occurance\n  - Ken Griffey\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {

                    let mut found_hr = false;
                    let mut found_rbi = false;
                    for (k, v) in pairs {
                        if let Node::Str(ks, _, _) = k {
                            if ks == "hr" {
                                if let Node::Array(items) = v {
                                    assert_eq!(items.len(), 2);
                                    assert_eq!(
                                        items[0],
                                        Node::Str(
                                            "Mark McGwire".to_string(),
                                            QuoteType::Unquoted,
                                            BlockStyle::None
                                        )
                                    );
                                    assert_eq!(
                                        items[1],
                                        Node::Str(
                                            "Sammy Sosa".to_string(),
                                            QuoteType::Unquoted,
                                            BlockStyle::None
                                        )
                                    );
                                    found_hr = true;
                                }
                            }
                            if ks == "rbi" {
                                if let Node::Array(items) = v {
                                    assert_eq!(items.len(), 2);
                                    assert_eq!(
                                        items[0],
                                        Node::Str(
                                            "Sammy Sosa".to_string(),
                                            QuoteType::Unquoted,
                                            BlockStyle::None
                                        )
                                    );
                                    assert_eq!(
                                        items[1],
                                        Node::Str(
                                            "Ken Griffey".to_string(),
                                            QuoteType::Unquoted,
                                            BlockStyle::None
                                        )
                                    );
                                    found_rbi = true;
                                }
                            }
                        }
                    }
                    assert!(found_hr && found_rbi);
                    return;
                }
            }
        }
        panic!("Unexpected parse result structure for hr/rbi anchor test");
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
                    assert_eq!(
                        items1[0],
                        Node::Str(
                            "2001-07-23".to_string(),
                            QuoteType::Unquoted,
                            BlockStyle::None
                        )
                    );
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
                    assert_eq!(
                        items2[0],
                        Node::Str(
                            "2001-07-02".to_string(),
                            QuoteType::Unquoted,
                            BlockStyle::None
                        )
                    );
                    assert_eq!(
                        items2[1],
                        Node::Str(
                            "2001-08-12".to_string(),
                            QuoteType::Unquoted,
                            BlockStyle::None
                        )
                    );
                    assert_eq!(
                        items2[2],
                        Node::Str(
                            "2001-08-14".to_string(),
                            QuoteType::Unquoted,
                            BlockStyle::None
                        )
                    );
                } else {
                    panic!("Second value is not an array");
                }

                return;
            }
        }
        panic!("Unexpected parse result structure for testfile017 explicit keys");
    }
    #[test]
    fn test_parse_block_unquoted_block_scalar_with_indent() {

        use crate::stringify;
        let yaml = b"---\nplain:\n  This unquoted scalar\n  spans many lines.";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        let mut dest = BufferDestination::new();
        stringify(&result, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "---\nplain: This unquoted scalar spans many lines.\n...\n"
        )
    }

    #[test]
    fn test_parse_block_double_quoted_block_scalar_with_indent() {

        use crate::stringify;
        let yaml = b"---\nquoted: \"So does this\n  quoted scalar.\\n\"";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        let mut dest = BufferDestination::new();
        stringify(&result, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "---\nquoted: |\n  So does this quoted scalar.\n...\n"
        )
    }
    #[test]
    fn test_parse_block_unquoted_block_multiline_scalar_with_indent() {

        use crate::stringify;
        let yaml = b"--- >\n Sammy Sosa completed another\n fine season with great stats.\n\n   63 Home Runs\n   0.288 Batting Average\n\n What a year!";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        let mut dest = BufferDestination::new();
        stringify(&result, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "--- |\nSammy Sosa completed another fine season with great stats.\n\n  63 Home Runs\n  0.288 Batting Average\n\nWhat a year!\n...\n"
        )
    }

    #[test]
    fn test_parse_block_multiline_scalars_with_indent() {

        use crate::stringify;
        let yaml = b"string1: |\n  Line1\n  line2\n  \"line3\"\n  line4\n\nstring2: >\n  Line1\n  line2\n  \"line3\"\n  line4\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        let mut dest = BufferDestination::new();
        stringify(&result, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "---\nstring1: |\n  Line1\n  line2\n  \"line3\"\n  line4\nstring2: |\n  Line1 line2 \"line3\" line4\n...\n"
        );
    }
    #[test]
    fn test_parse_escapes_in_strings() {

        use crate::stringify;
        let yaml = b"unicode: \"Sosa did fine.\\u263A\"\ncontrol: \"\\b1998\\t1999\\t2000\\n\"\nhexesc:  \"\\x13\\x10 is \\r\\n\"\n\nsingle: \'\"Howdy!\" he cried.\'\nquoted: \' # not a \'\'comment\'\'.\'\ntie-fighter: \'|\\-*-/|\'\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        let mut dest = BufferDestination::new();
        stringify(&result, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "---\nunicode: Sosa did fine.☺\ncontrol: \"\\b1998\\t1999\\t2000\\n\"\nhexesc: \"\\x13\\x10 is \\r\\n\"\nsingle: \'\"Howdy!\" he cried.\'\nquoted: \" # not a \'comment\'.\"\ntie-fighter: \"|\\\\-*-/|\"\n...\n"
        );
    }

    #[test]
    fn test_parse_mapping_with_quoted_string_value() {
        let yaml =
            b"\'Keys can be quoted too.\': \"Useful if you want to put a \':\' in your key.\"";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        let mut dest = BufferDestination::new();
        crate::stringify(&result, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "---\nKeys can be quoted too.: Useful if you want to put a ':' in your key.\n...\n"
        );
    }
    #[test]
    fn test_parse_a_literal_block_scalar() {
        let yaml = b"literal_block: |\n  This entire block of text will be the value of the \'literal_block\' key,\n  with line breaks being preserved.\n\n  The literal continues until de-dented, and the leading indentation is\n  stripped.\n\n      Any lines that are \'more-indented\' keep the rest of their indentation -\n      these lines will be indented by 4 spaces.";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        let mut dest = BufferDestination::new();
        crate::stringify(&result, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "---\nliteral_block: |\n  This entire block of text will be the value of the \'literal_block\' key,\n  with line breaks being preserved.\n\n  The literal continues until de-dented, and the leading indentation is\n  stripped.\n\n      Any lines that are \'more-indented\' keep the rest of their indentation -\n      these lines will be indented by 4 spaces.\n...\n"
        );
    }
    #[test]
    fn test_parse_a_literal_scalar_strip() {
        let yaml = b"literal_strip: |-\n  This entire block of text will be the value of the \'literal_strip\' key,\n  with trailing blank line being stripped.";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        let mut dest = BufferDestination::new();
        crate::stringify(&result, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "---\nliteral_strip: |\n  This entire block of text will be the value of the \'literal_strip\' key,\n  with trailing blank line being stripped.\n...\n"
        );
    }
    #[test]
    fn test_parse_a_block_scalar_strip() {
        let yaml = b"block_strip: >-\n  This entire block of text will be the value of \'block_strip\', but this\n  time, all newlines will be replaced with a single space and\n  trailing blank line being stripped.\n\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        let mut dest = BufferDestination::new();
        crate::stringify(&result, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "---\nblock_strip: |\n  This entire block of text will be the value of \'block_strip\', but this time, all newlines will be replaced with a single space and trailing blank line being stripped.\n...\n"
        );
    }


    #[test]
    fn test_parse_nested_anchor_and_alias_with_block_scalar() {


        let yaml = b"base: &base\n  name: Everyone has same name\nfoo:\n  <<: *base\n  age: 10\n  name: John\nbar:\n  <<: *base\n  age: 20";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();


        let mut mutated = result.clone();
        use std::collections::HashMap as Map;


        let mut anchors: Map<String, Node> = Map::new();
        if let Node::Documents(docs) = &mut mutated {
            if let Node::Document(nodes) = &mut docs[0] {
                if let Node::Mapping(pairs) = &mut nodes[0] {
                    for (_k, v) in pairs.iter_mut() {
                        if let Node::Anchored(inner, name) = v {

                            let unboxed: Node = (**inner).clone();
                            anchors.insert(name.clone(), unboxed.clone());

                            *v = unboxed;
                        }
                    }


                    let mut anchor_map: Map<String, Vec<(Node, Node)>> = Map::new();
                    {
                        let snapshot = pairs.clone();
                        for (k, v) in snapshot.iter() {
                            if let Node::Str(ks, _, _) = k {
                                if let Node::Mapping(ap) = v {
                                    anchor_map.insert(ks.clone(), ap.clone());
                                }
                            }
                        }
                    }


                    let mut rebuilt: Vec<(Node, Node)> = Vec::new();
                    let mut i = 0usize;
                    while i < pairs.len() {
                        let (k, v) = pairs[i].clone();
                        if let Node::Str(_, _, _) = &k {
                            if let Node::Str(s, _, _) = &v {

                                if s.trim_start().starts_with("<<:") && s.contains('*') {

                                    if let Some(pos) = s.find('*') {
                                        let aname = s[pos + 1..].trim().to_string();

                                        let mut nested: Vec<(Node, Node)> = Vec::new();
                                        let mut j = i + 1;
                                        while j < pairs.len() {

                                            if let Node::Str(ns, _, _) = &pairs[j].1 {
                                                if ns.trim_start().starts_with("<<:") {
                                                    break;
                                                }
                                            }
                                            nested.push(pairs[j].clone());
                                            j += 1;
                                        }


                                        let mut merged_pairs: Vec<(Node, Node)> = Vec::new();
                                        if let Some(ap) = anchor_map.get(&aname) {
                                            merged_pairs.extend(ap.clone());
                                        }

                                        for (nk, nv) in nested.iter() {

                                            let mut replaced = false;
                                            if let Node::Str(nks, _, _) = nk {
                                                for p in merged_pairs.iter_mut() {
                                                    if let Node::Str(pk, _, _) = &p.0 {
                                                        if pk == nks {
                                                            *p = (nk.clone(), nv.clone());
                                                            replaced = true;
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            if !replaced {
                                                merged_pairs.push((nk.clone(), nv.clone()));
                                            }
                                        }


                                        rebuilt.push((k.clone(), Node::Mapping(merged_pairs)));
                                        i = j;
                                        continue;
                                    }
                                }
                            }
                        }

                        rebuilt.push((k, v));
                        i += 1;
                    }

                    *pairs = rebuilt;
                }
            }
        }


        let mut dest = BufferDestination::new();
        crate::stringify(&mutated, &mut dest).unwrap();
        let out = dest.to_string();
        assert!(
            !out.contains("<<:"),
            "merge keys should be expanded and removed: {}",
            out
        );
        assert!(
            out.contains("base: \n  name: Everyone has same name"),
            "base mapping missing or altered: {}",
            out
        );
        assert!(
            out.contains("foo: \n  name: John\n  age: 10"),
            "foo mapping not merged correctly: {}",
            out
        );
        assert!(
            out.contains("bar: \n  name: Everyone has same name\n  age: 20"),
            "bar mapping not merged correctly: {}",
            out
        );
    }

    static TEST_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TestFile {
        path: String,
    }

    impl TestFile {
        fn new_with_content(content: &[u8]) -> Self {
            let id = TEST_FILE_COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = format!("test_stringify_temp_{}.yaml", id);
            fs::write(&path, content).unwrap();
            Self { path }
        }

        fn new_empty() -> Self {
            let id = TEST_FILE_COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = format!("test_stringify_temp_{}.yaml", id);
            fs::write(&path, b"").unwrap();
            Self { path }
        }

        fn path(&self) -> &str {
            &self.path
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    #[test]
    fn test_stringify_none() {
        let mut dest = BufferDestination::new();
        stringify(&Node::None, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "null");
    }

    #[test]
    fn test_stringify_boolean() {
        let mut dest = BufferDestination::new();
        stringify(&Node::Boolean(true), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "true");
    }

    #[test]
    fn test_stringify_string() {
        let mut dest = BufferDestination::new();
        stringify(
            &Node::Str("test".to_string(), QuoteType::Double, BlockStyle::None),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "\"test\"");
    }

    #[test]
    fn test_stringify_comment() {
        let mut dest = BufferDestination::new();
        stringify(&Node::Comment("test".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "# test");
    }

    #[test]
    fn test_stringify_numbers() {
        let mut dest = BufferDestination::new();
        stringify(&Node::Number(Numeric::Integer(42)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "42");

        dest = BufferDestination::new();
        stringify(&Node::Number(Numeric::Float(3.14)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "3.14");
    }

    #[test]
    fn test_stringify_array() {
        let mut dest = BufferDestination::new();
        let arr = vec![
            Node::Number(Numeric::Integer(1)),
            Node::Str("test".to_string(), QuoteType::Double, BlockStyle::None),
        ];
        stringify(&Node::Array(arr), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "- 1\n- \"test\"\n");
    }

    #[test]
    fn test_stringify_mapping() {
        let mut dest = BufferDestination::new();
        let mapping = Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Double, BlockStyle::None),
            Node::Str("value".to_string(), QuoteType::Double, BlockStyle::None),
        )]);
        stringify(&mapping, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"key\": \"value\"\n");
    }

    #[test]
    fn test_stringify_documents() {
        let mut dest = BufferDestination::new();
        let docs = vec![
            Node::Str("doc1".to_string(), QuoteType::Double, BlockStyle::None),
            Node::Str("doc2".to_string(), QuoteType::Double, BlockStyle::None),
        ];
        stringify(&Node::Documents(docs), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "---\n\"doc1\"...\n---\n\"doc2\"...\n");
    }

    #[test]
    fn test_stringify_anchor_and_alias() {
        let mut dest = BufferDestination::new();

        let anchored = Node::Anchored(
            Box::new(Node::Mapping(vec![(
                Node::Str("nested".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
            )])),
            "a".to_string(),
        );

        let docs = vec![anchored, Node::Alias("a".to_string())];
        stringify(&Node::Documents(docs), &mut dest).unwrap();

        let out = dest.to_string();
        assert!(out.contains("&a"));
        assert!(out.contains("*a"));
    }

    #[test]
    fn test_stringify_integer_sequence() {
        let mut dest = BufferDestination::new();
        let mut source = BufferSource::new("---\n- 1\n- 2\n- 3\n...\n".as_bytes());
        let node = parse(&mut source).unwrap();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "---\n- 1\n- 2\n- 3\n...\n");
    }
    #[test]
    fn test_stringify_sequence_with_nested_mapping() {
        let mut dest = BufferDestination::new();
        let mut source = BufferSource::new("---\n- \n  name: Mark Joseph\n  hr: 87\n  avg: 0.278\n- \n  name: James Stephen\n  hr: 63\n  avg: 0.288\n...\n".as_bytes());
        let node = parse(&mut source).unwrap();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "---\n- name: Mark Joseph\n  hr: 87\n  avg: 0.278\n- name: James Stephen\n  hr: 63\n  avg: 0.288\n...\n"
        );
    }
    #[test]
    fn test_stringify_sequence_with_nested_sequence() {
        let mut dest = BufferDestination::new();
        let mut source = BufferSource::new("- [Sammy Sosa, 63, 0.288]".as_bytes());
        let node = parse(&mut source).unwrap();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "---\n- - Sammy Sosa\n  - 63\n  - 0.288\n...\n"
        );
    }

    #[test]
    fn test_stringify_anchor_alias_hr_rbi() {
        use crate::io::sources::buffer::Buffer as SrcBuffer;

        let yaml = b"---\nhr:\n  - Mark McGwire\n  # Following node labeled SS\n  - &SS Sammy Sosa\nrbi:\n  - *SS # Subsequent occurance\n  - Ken Griffey\n";
        let mut source = SrcBuffer::new(yaml);
        let node = parse(&mut source).unwrap();

        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        let expected = "---\nhr: \n  - Mark McGwire\n  - Sammy Sosa\nrbi: \n  - Sammy Sosa\n  - Ken Griffey\n...\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn test_stringify_anchor_alias_hr_rbi_from_file() {
        use crate::io::sources::file::File as FileSource;


        let input = b"---\r\nhr:\r\n  - Mark McGwire\r\n  # Following node labeled SS\r\n  - &SS Sammy Sosa\r\nrbi:\r\n  - *SS # Subsequent occurance\r\n  - Ken Griffey\r\n";
        let in_file = TestFile::new_with_content(input);


        let mut source = FileSource::new(in_file.path()).unwrap();
        let node = parse(&mut source).unwrap();


        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        let expected = "---\nhr: \n  - Mark McGwire\n  - Sammy Sosa\nrbi: \n  - Sammy Sosa\n  - Ken Griffey\n...\n";
        assert_eq!(out, expected);
    }
    #[test]
    fn test_with_comment_header() {
        let mut dest = BufferDestination::new();
        let mut source = BufferSource::new(
            "# Ranking of 1998 home runs\n---\n- Mark Joseph\n- James Stephen\n- Ken Griffey\n"
                .as_bytes(),
        );
        let node = parse(&mut source).unwrap();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "---\n- Mark Joseph\n- James Stephen\n- Ken Griffey\n...\n"
        );
    }

    #[test]
    fn test_stringify_double_quoted_multiline_scalar() {
        let mut dest = BufferDestination::new();
        let node = Node::Str(
            "line1\nline2".to_string(),
            QuoteType::Double,
            BlockStyle::None,
        );
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"line1\nline2\"");
    }

    #[test]
    fn test_stringify_single_quoted_multiline_and_escaping() {
        let mut dest = BufferDestination::new();
        let node = Node::Str(
            "O'Reilly\nBooks".to_string(),
            QuoteType::Single,
            BlockStyle::None,
        );
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "'O''Reilly\nBooks'");
    }

    #[test]
    fn test_stringify_mapping_with_multiline_value() {
        let mut dest = BufferDestination::new();
        let mapping = Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("a\nb".to_string(), QuoteType::Double, BlockStyle::None),
        )]);
        stringify(&mapping, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "key: \"a\nb\"\n");
    }

    #[test]
    fn test_stringify_sequence_with_multiline_item() {
        let mut dest = BufferDestination::new();
        let seq = Node::Array(vec![Node::Str(
            "a\nb".to_string(),
            QuoteType::Double,
            BlockStyle::None,
        )]);
        stringify(&seq, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "- \"a\nb\"\n");
    }

    #[test]
    fn test_stringify_mapping_with_inline_comment_and_indented_sequence_files() {

        use crate::io::destinations::file::File as FileDestination;
        use crate::io::sources::file::File as FileSource;
        let input = b"---\r\nhr: # 1998 hr ranking\r\n  - Mark McGwire\r\n  - Sammy Sosa\n";
        let in_file = TestFile::new_with_content(input);
        let out_file = TestFile::new_empty();

        let mut source = FileSource::new(in_file.path()).unwrap();
        let node = parse(&mut source).unwrap();

        let mut dest = FileDestination::new(out_file.path()).unwrap();
        stringify(&node, &mut dest).unwrap();

        let out = fs::read_to_string(out_file.path()).unwrap();
        let expected = "---\nhr: \n  - Mark McGwire\n  - Sammy Sosa\n...\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn test_stringify_explicit_sequence_keys() {

        let yaml = b"? # PLAY SCHEDULE\n  - Detroit Tigers\n  - Chicago Cubs\n:\n  - 2001-07-23\n\n? [ New York Yankees,\n    Atlanta Braves ]\n: [ 2001-07-02, 2001-08-12,\n    2001-08-14 ]\n";
        let mut src = BufferSource::new(yaml.as_ref());
        let node = parse(&mut src).expect("parse");

        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).expect("");
        let out = normalize_newlines(&dest.to_string());

        let expected = normalize_newlines(
            "---\n\"[Detroit Tigers, Chicago Cubs]\": \n  - 2001-07-23\n\"[New York Yankees, Atlanta Braves]\": \n  - 2001-07-02\n  - 2001-08-12\n  - 2001-08-14\n...\n",
        );

        assert_eq!(out, expected);
    }


    #[test]
    fn test_stringify_from_parser_literal_block_literal_scalar_with_indent() {
        let yaml = b"---\nstring1: |\n  Line1\n  line2\n";
        let mut source = BufferSource::new(yaml);
        let node = parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        assert_eq!(out, "---\nstring1: |\n  Line1\n  line2\n...\n");
    }

    #[test]
    fn test_stringify_from_parser_literal_block_folded_scalar_with_indent() {
        let yaml = b"---\nstring1: >\n  Line1\n  line2\n";
        let mut source = BufferSource::new(yaml);
        let node = parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();

        assert_eq!(out, "---\nstring1: |\n  Line1 line2\n...\n");
    }

    #[test]
    fn test_stringify_merge_key_with_single_alias() {
        let yaml = b"---\na: &a\n  nested: anchor\nparent:\n  <<: *a\n  key: value\n";
        let mut source = BufferSource::new(yaml);
        let node = parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();


        assert!(
            !out.contains("<<:"),
            "stringified output should not contain merge key: {}",
            out
        );
        assert!(out.contains("parent:"));
        assert!(out.contains("nested: anchor"));
    }
    #[test]
    fn test_parse_stringify_floating_point_key() {
        let yaml = b"---\n0.25: a float key\n";
        let mut source = BufferSource::new(yaml);
        let node = parse(&mut source).unwrap();

        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        assert_eq!(out, "---\n\"0.25\": a float key\n...\n");
    }
    #[test]
    fn test_parse_stringify_multi_line_string_key() {
        let yaml = b"? |\n This is a key\n that has multiple lines\n : and this is its value";
        let mut source = BufferSource::new(yaml);
        let node = parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        assert_eq!(
            out,
            "---\n\"This is a key\\nthat has multiple lines\\n\": and this is its value\n...\n"
        );
    }
    #[test]
    fn test_parse_stringify_multi_line_sequence_string_key() {
        let yaml = b"? - Manchester United\n  - Real Madrid\n : [ 2001-01-01, 2002-02-02 ]\n";
        let mut source = BufferSource::new(yaml);
        let node = parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        assert_eq!(
            out,
            "---\n\"[Manchester United, Real Madrid]\": \n  - 2001-01-01\n  - 2002-02-02\n...\n"
        );
    }
        #[test]
    fn test_parse_stringify_mapping_and_inent() {
        let yaml = b"base: &base\n name: Everyone has same name\nfoo:\n <<: *base # doesn\'t merge the anchor\n age: 10\n name: John\nbar:\n <<: *base # base anchor will be merged\n age: 20\n\nexplicit_boolean: !!bool true\n";
        let mut source = BufferSource::new(yaml);
        let node = parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        assert_eq!(
            out,
            "---\nbase: \n  name: Everyone has same name\nfoo: \n  name: John\n  age: 10\nbar: \n  name: Everyone has same name\n  age: 20\nexplicit_boolean: true\n...\n"
        );
    }
}
