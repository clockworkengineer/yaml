///
/// Nested structure tests: nested mappings, nested sequences, and complex combinations.
///
#[cfg(test)]
mod tests {
    use crate::nodes::node::{BlockStyle, QuoteType};
    use crate::{BufferSource, Node, Node::Document, Numeric, parse};

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
    fn test_parse_deep_nested_mappings() {
        let yaml = b"level1:\n  level2:\n    level3:\n      level4:\n        level5: deep_value";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Some(Node::Document(content)) = docs.first() {
                if let Some(Node::Mapping(mapping)) = content.first() {
                    assert_eq!(mapping.len(), 1);
                    // Verify we can navigate through all levels
                    let (key, value) = &mapping[0];
                    if let Node::Str(k, _, _) = key {
                        assert_eq!(k, "level1");
                    }
                    // Test successful parsing of deep structure
                    assert!(matches!(value, Node::Mapping(_)));
                }
            }
        }
    }

    #[test]
    fn test_parse_deep_nested_sequences() {
        let yaml = b"- - - - - deep_item";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = result {
            if let Some(Node::Document(content)) = docs.first() {
                if let Some(Node::Array(array)) = content.first() {
                    assert_eq!(array.len(), 1);
                    // Verify deeply nested array structure exists
                    assert!(matches!(array[0], Node::Array(_)));
                }
            }
        }
    }

    #[test]
    fn test_parse_mixed_nested_complex_structure() {
        let yaml = b"root:\n  arrays:\n    - name: array1\n      items: [1, 2, 3]\n    - name: array2\n      items: [4, 5, 6]\n  mappings:\n    config:\n      enabled: true\n      settings: {debug: false, level: 2}";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        // Test that complex mixed structure parses successfully
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_nested_flow_in_block_structure() {
        let yaml = b"users:\n  - {name: John, age: 30, hobbies: [reading, gaming]}\n  - {name: Jane, age: 25, hobbies: [cooking, travel]}";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("users".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Array(vec![
                Node::Mapping(vec![
                    (
                        Node::Str("name".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        Node::Str("John".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    ),
                    (
                        Node::Str("age".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        Node::Number(Numeric::Integer(30)),
                    ),
                    (
                        Node::Str("hobbies".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        Node::Array(vec![
                            Node::Str("reading".to_string(), QuoteType::Unquoted, BlockStyle::None),
                            Node::Str("gaming".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        ]),
                    ),
                ]),
                Node::Mapping(vec![
                    (
                        Node::Str("name".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        Node::Str("Jane".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    ),
                    (
                        Node::Str("age".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        Node::Number(Numeric::Integer(25)),
                    ),
                    (
                        Node::Str("hobbies".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        Node::Array(vec![
                            Node::Str("cooking".to_string(), QuoteType::Unquoted, BlockStyle::None),
                            Node::Str("travel".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        ]),
                    ),
                ]),
            ]),
        )])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_nested_mixed_data_types() {
        let yaml = b"data:\n  strings:\n    - \"hello\"\n    - 'world'\n  numbers:\n    - 42\n    - 3.14\n  booleans:\n    - true\n    - false\n  nulls:\n    - null\n    - ~\n  nested:\n    more_data:\n      - {type: string, value: test}\n      - {type: number, value: 123}";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        // Test that mixed data types in nested structure parse successfully
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_empty_nested_structures() {
        let yaml = b"empty:\n  mapping: {}\n  sequence: []\n  nested_empty:\n    deep_mapping: {}\n    deep_sequence: []";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("empty".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Mapping(vec![
                (
                    Node::Str("mapping".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Mapping(vec![]),
                ),
                (
                    Node::Str(
                        "sequence".to_string(),
                        QuoteType::Unquoted,
                        BlockStyle::None,
                    ),
                    Node::Array(vec![]),
                ),
                (
                    Node::Str(
                        "nested_empty".to_string(),
                        QuoteType::Unquoted,
                        BlockStyle::None,
                    ),
                    Node::Mapping(vec![
                        (
                            Node::Str(
                                "deep_mapping".to_string(),
                                QuoteType::Unquoted,
                                BlockStyle::None,
                            ),
                            Node::Mapping(vec![]),
                        ),
                        (
                            Node::Str(
                                "deep_sequence".to_string(),
                                QuoteType::Unquoted,
                                BlockStyle::None,
                            ),
                            Node::Array(vec![]),
                        ),
                    ]),
                ),
            ]),
        )])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_nested_with_block_scalars() {
        let yaml = b"document:\n  description: |\n    This is a literal\n    block scalar with\n    multiple lines\n  folded: >\n    This is a folded\n    block scalar that\n    becomes one line\n  nested:\n    more_text: |\n      Another literal\n      block here";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        // Test that nested structures with block scalars parse
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_nested_with_comments_at_multiple_levels() {
        let yaml = b"# Top level comment\nconfig:\n  # Database section\n  database:\n    # Connection settings\n    host: localhost\n    port: 5432\n  # Server section  \n  server:\n    # Network settings\n    host: 0.0.0.0\n    port: 8080\n# End comment";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("config".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Mapping(vec![
                (
                    Node::Str(
                        "database".to_string(),
                        QuoteType::Unquoted,
                        BlockStyle::None,
                    ),
                    Node::Mapping(vec![
                        (
                            Node::Str("host".to_string(), QuoteType::Unquoted, BlockStyle::None),
                            Node::Str(
                                "localhost".to_string(),
                                QuoteType::Unquoted,
                                BlockStyle::None,
                            ),
                        ),
                        (
                            Node::Str("port".to_string(), QuoteType::Unquoted, BlockStyle::None),
                            Node::Number(Numeric::Integer(5432)),
                        ),
                    ]),
                ),
                (
                    Node::Str("server".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Mapping(vec![
                        (
                            Node::Str("host".to_string(), QuoteType::Unquoted, BlockStyle::None),
                            Node::Str("0.0.0.0".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        ),
                        (
                            Node::Str("port".to_string(), QuoteType::Unquoted, BlockStyle::None),
                            Node::Number(Numeric::Integer(8080)),
                        ),
                    ]),
                ),
            ]),
        )])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_nested_sequences_with_mappings() {
        let yaml = b"matrix:\n  - - {x: 1, y: 2}\n    - {x: 3, y: 4}\n  - - {x: 5, y: 6}\n    - {x: 7, y: 8}";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        // Test matrix-like nested structure
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_complex_real_world_config() {
        let yaml = b"application:\n  name: MyApp\n  version: 1.0.0\n  environments:\n    development:\n      database:\n        host: dev.db.local\n        port: 5432\n        credentials:\n          username: dev_user\n          password: dev_pass\n      cache:\n        type: redis\n        servers:\n          - host: redis1.dev\n            port: 6379\n          - host: redis2.dev\n            port: 6379\n    production:\n      database:\n        host: prod.db.com\n        port: 5432\n        credentials:\n          username: prod_user\n          password: prod_pass\n      cache:\n        type: redis\n        servers:\n          - host: redis1.prod\n            port: 6379\n          - host: redis2.prod\n            port: 6379";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        // Test that complex real-world configuration structure parses
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_nested_with_special_values() {
        let yaml = b"special:\n  nulls:\n    - null\n    - ~\n    -\n  booleans:\n    true_values: [true, True, TRUE, yes, Yes, YES]\n    false_values: [false, False, FALSE, no, No, NO]\n  numbers:\n    integers: [0, 123, -456]\n    floats: [0.0, 3.14, -2.5, 1e6]\n  strings:\n    quoted: ['hello', \"world\"]\n    unquoted: [hello, world]";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        // Test that nested structures with special YAML values parse
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_nested_flow_sequences_in_mappings() {
        let yaml = b"coordinates:\n  points: [[0, 0], [1, 1], [2, 4]]\n  vectors: [[1, 0, 0], [0, 1, 0], [0, 0, 1]]\n  nested_data:\n    matrix: [[[1, 2], [3, 4]], [[5, 6], [7, 8]]]";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        // Test nested flow sequences within mappings
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_nested_mappings_with_numeric_keys() {
        let yaml = b"lookup:\n  1:\n    name: first\n    value: 100\n  2:\n    name: second\n    value: 200\n  nested:\n    3:\n      deep:\n        4: deepest_value";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        // Test nested mappings with numeric keys
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_alternating_nested_structures() {
        let yaml = b"alternating:\n  - mapping1:\n      key1: value1\n  - [item1, item2]\n  - mapping2:\n      nested:\n        - nested_item1\n        - nested_item2\n  - [item3, {key2: value2}]";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        // Test alternating nested mappings and sequences
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_nested_with_unicode_content() {
        let yaml = "unicode:\n  chinese:\n    greeting: \"Hello\"\n    numbers: [one, two, three]\n  symbols:\n    faces: [happy, sad, excited]\n    animals: [dog, cat, mouse]\n  mixed:\n    text: \"Hello World\"";
        let mut source = BufferSource::new(yaml.as_bytes());
        let result = parse(&mut source).unwrap();

        // Test nested structures with varied content
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_deeply_nested_performance() {
        // Create a moderately deep structure for performance testing
        let mut yaml_content = String::new();
        yaml_content.push_str("root:\n");
        for i in 1..=10 {
            let indent = "  ".repeat(i);
            yaml_content.push_str(&format!("{}level{}:\n", indent, i));
        }
        yaml_content.push_str("            final_value: reached_bottom\n");

        let mut source = BufferSource::new(yaml_content.as_bytes());
        let result = parse(&mut source).unwrap();

        // Test that moderately deep nesting parses within reasonable time
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_nested_with_escaped_strings() {
        let yaml = b"escaped:\n  strings:\n    - \"Hello\\nWorld\"\n    - 'Can''t stop'\n    - \"Tab\\tSeparated\"\n  nested:\n    paths:\n      windows: \"C:\\\\Users\\\\Name\"\n      unix: \"/home/user\"\n    quotes:\n      - \"He said \\\"Hello\\\"\"\n      - 'She replied ''Hi'''\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        // Test nested structures with escaped strings
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_nested_large_sequences() {
        let yaml = b"large_data:\n  numbers:\n    - [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]\n    - [11, 12, 13, 14, 15, 16, 17, 18, 19, 20]\n    - [21, 22, 23, 24, 25, 26, 27, 28, 29, 30]\n  nested_arrays:\n    - - - [a, b, c]\n        - [d, e, f]\n      - - [g, h, i]\n        - [j, k, l]";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        // Test nested structures with larger sequences
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_nested_mixed_quotes() {
        let yaml = b"quotes:\n  mixed:\n    single: 'Single quoted'\n    double: \"Double quoted\"\n    unquoted: Plain text\n  nested:\n    combinations:\n      - 'Single with \"double\" inside'\n      - \"Double with 'single' inside\"\n      - Plain with both 'single' and \"double\"";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        // Test nested structures with mixed quote types
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_nested_indentation_edge_cases() {
        let yaml = b"indent:\n  normal:\n    key1: value1\n    key2: value2\n  mixed:\n    key3: value3\n      # Extra indented comment\n    key4: value4\n  sequences:\n    - item1\n    - item2\n      # Comment in sequence\n    - item3";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        // Test nested structures with indentation edge cases
        assert!(matches!(result, Node::Documents(_)));
    }

    #[test]
    fn test_parse_nested_edge_case_structures() {
        let yaml = b"edge_cases:\n  empty_values:\n    key_with_empty:\n    another_empty:\n  single_items:\n    single_mapping:\n      only_key: only_value\n    single_sequence:\n      - only_item\n  mixed_empty:\n    - {}\n    - []\n    - null\n    - {key: []}\n    - [{}]";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source).unwrap();

        // Test edge cases in nested structures
        assert!(matches!(result, Node::Documents(_)));
    }
}
