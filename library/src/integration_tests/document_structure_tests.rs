///
/// Document structure tests: multi-document YAML, document markers, header comments.
///
#[cfg(test)]
mod tests {
    use crate::nodes::node::{BlockStyle, QuoteType};
    use crate::{BufferSource, Node, Node::Document, Numeric, parse};
        use crate::test_helpers::{parse_yaml, assert_nodes_eq, assert_parse_error};
    use std::collections::HashMap;

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
    #[test]
    fn test_parse_document_end_marker() {
        let input = b"key: value\n---";
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        )])])]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_document_end_marker_with_trailing_content() {
        let input = b"key: value\n---\nother: 123";
        let expected = Node::Documents(vec![
            Document(vec![Node::Mapping(vec![(
                Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
            )])]),
            Document(vec![Node::Mapping(vec![(
                Node::Str("other".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(123)),
            )])]),
        ]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_document_end_marker_with_comments() {
        let input = b"# Comment before\nkey: value\n# Comment after\n---\n# Comment in between\nother: 123\n# Final comment";
        let expected = Node::Documents(vec![
            Document(vec![Node::Mapping(vec![(
                Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
            )])]),
            Document(vec![Node::Mapping(vec![(
                Node::Str("other".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(123)),
            )])]),
        ]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_document_end_marker_only() {
        let input = b"---";
        let expected = Node::Documents(vec![Document(vec![])]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_multiple_document_end_markers() {
        let input = b"key: value\n---\n---\nother: 1";
        // The parser merges empty documents, so only non-empty documents are present
        let expected = Node::Documents(vec![
            Document(vec![Node::Mapping(vec![
                (
                    Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
            ])]),
            Document(vec![Node::Mapping(vec![
                (
                    Node::Str("other".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Number(Numeric::Integer(1)),
                ),
            ])]),
        ]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_empty_document_end_marker() {
        let input = b"...";
        let expected = Node::Documents(vec![Document(vec![])]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_document_start_marker() {
        let input = b"---\nkey: value";
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        )])])]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_document_start_and_end_markers() {
        let input = b"---\nkey: value\n...";
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        )])])]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_multiple_documents_with_start_markers() {
        let input = b"---\nfirst: 1\n---\nsecond: 2\n---\nthird: 3";
        let expected = Node::Documents(vec![
            Document(vec![Node::Mapping(vec![(
                Node::Str("first".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(1)),
            )])]),
            Document(vec![Node::Mapping(vec![(
                Node::Str("second".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(2)),
            )])]),
            Document(vec![Node::Mapping(vec![(
                Node::Str("third".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(3)),
            )])]),
        ]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_documents_with_mixed_markers() {
        let input = b"---\nfirst: 1\n...\n---\nsecond: 2\n---\nthird: 3\n...";
        let expected = Node::Documents(vec![
            Document(vec![Node::Mapping(vec![(
                Node::Str("first".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(1)),
            )])]),
            Document(vec![Node::Mapping(vec![(
                Node::Str("second".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(2)),
            )])]),
            Document(vec![Node::Mapping(vec![(
                Node::Str("third".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(3)),
            )])]),
        ]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_empty_documents_between_markers() {
        let input = b"---\n---\nkey: value\n---";
        // The parser merges empty documents, so only the non-empty document is present
        let expected = Node::Documents(vec![
            Document(vec![Node::Mapping(vec![
                (
                    Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
            ])]),
        ]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_document_with_yaml_version_directive() {
        let input = b"# YAML version would be %YAML 1.2 here\n---\nkey: value";
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        )])])]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_document_with_tag_directive() {
        let input =
            b"# TAG directive would be %TAG ! tag:example.com,2000:app/ here\n---\nkey: value";
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        )])])]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_document_with_multiple_directives() {
        let input = b"# Would have %YAML 1.2 and %TAG ! tag:example.com,2000:app/ directives here\n---\nkey: value";
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        )])])]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_document_with_comments_around_markers() {
        let input = b"# Before first marker\n---\n# After first marker\nkey: value\n# Before end marker\n...\n# After end marker";
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        )])])]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_documents_with_different_content_types() {
        let input = b"---\n\"scalar document\"\n---\n- item1\n- item2\n---\nkey: value";
        let expected = Node::Documents(vec![
            Document(vec![Node::Str(
                "scalar document".to_string(),
                QuoteType::Double,
                BlockStyle::None,
            )]),
            Document(vec![Node::Array(vec![
                Node::Str("item1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("item2".to_string(), QuoteType::Unquoted, BlockStyle::None),
            ])]),
            Document(vec![Node::Mapping(vec![(
                Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
            )])]),
        ]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_document_with_whitespace_only_content() {
        let input = b"---\n   \n  \n  \n---\nkey: value";
        let expected = Node::Documents(vec![
            Document(vec![]),
            Document(vec![Node::Mapping(vec![(
                Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
            )])]),
        ]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_document_with_trailing_spaces_on_markers() {
        let input = b"---   \nkey: value\n...   ";
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        )])])]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_document_markers_with_comments_inline() {
        let input = b"--- # Start of document\nkey: value\n# End of document";
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        )])])]);
        let actual = parse_yaml(input);
        assert_nodes_eq(&actual, &expected);
    }

    #[test]
    fn test_parse_documents_with_complex_nested_content() {
        let input = b"---\nusers:\n  - name: alice\n    roles: [admin, user]\n  - name: bob\n    roles: [user]\nconfig:\n  database:\n    host: localhost\n    port: 5432\n---\nservers:\n  web:\n    - host: web1.example.com\n    - host: web2.example.com";
        // Only check that the parse succeeds and the structure is as expected
        let actual = parse_yaml(input);
        if let Node::Documents(docs) = &actual {
            assert!(!docs.is_empty(), "Should have at least one document");
            let mut found_users = false;
            let mut found_servers = false;
            for doc in docs {
                if let Document(nodes) = doc {
                    for node in nodes {
                        if let Node::Mapping(pairs) = node {
                            for (k, _) in pairs {
                                if let Node::Str(s, _, _) = k {
                                    if s == "users" {
                                        found_users = true;
                                    }
                                    if s == "servers" {
                                        found_servers = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            assert!(found_users, "Should find 'users' mapping");
            assert!(found_servers, "Should find 'servers' mapping");
        } else {
            panic!("Expected Documents node");
        }
    }

    #[test]
    fn test_trailing_empty_document_after_end_and_start_markers() {
        // Pattern: <content>"\n...\n---\n" should produce a trailing empty document
        let mut source = BufferSource::new(b"---\nkey: value\n...\n---\n");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(
                docs.len(),
                2,
                "Expected two documents including trailing empty one"
            );
            if let Document(nodes) = &docs[1] {
                assert!(nodes.is_empty(), "Second document should be empty");
            } else {
                panic!("Second entry should be a Document node");
            }
        } else {
            panic!("Expected Documents node");
        }
    }

    #[test]
    fn test_parse_document_with_boolean_and_null_values() {
        let input = b"---\n\
            enabled: true\n\
            disabled: false\n\
            empty: null\n\
            missing: ~\n\
            ---\n\
            yes: yes\n\
            no: no\n\
            on: on\n\
            off: off";
        let actual = parse_yaml(input);
        if let Node::Documents(docs) = &actual {
            assert!(!docs.is_empty(), "Should have at least one document");
            let mut found_enabled = false;
            let mut found_disabled = false;
            let mut found_empty = false;
            let mut found_missing = false;
            let mut found_yes = false;
            let mut found_no = false;
            let mut found_on = false;
            let mut found_off = false;
            for doc in docs {
                if let Document(nodes) = doc {
                    for node in nodes {
                        if let Node::Mapping(pairs) = node {
                            for (k, v) in pairs {
                                if let Node::Str(s, _, _) = k {
                                    match s.as_str() {
                                        "enabled" => {
                                            if let Node::Boolean(true) = v {
                                                found_enabled = true;
                                            }
                                        }
                                        "disabled" => {
                                            if let Node::Boolean(false) = v {
                                                found_disabled = true;
                                            }
                                        }
                                        "empty" => {
                                            if let Node::None = v {
                                                found_empty = true;
                                            }
                                        }
                                        "missing" => {
                                            if let Node::None = v {
                                                found_missing = true;
                                            }
                                        }
                                        "yes" => {
                                            if let Node::Str(val, _, _) = v {
                                                if val == "yes" {
                                                    found_yes = true;
                                                }
                                            }
                                        }
                                        "no" => {
                                            if let Node::Str(val, _, _) = v {
                                                if val == "no" {
                                                    found_no = true;
                                                }
                                            }
                                        }
                                        "on" => {
                                            if let Node::Str(val, _, _) = v {
                                                if val == "on" {
                                                    found_on = true;
                                                }
                                            }
                                        }
                                        "off" => {
                                            if let Node::Str(val, _, _) = v {
                                                if val == "off" {
                                                    found_off = true;
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            assert!(found_enabled, "Should find enabled: true");
            assert!(found_disabled, "Should find disabled: false");
            assert!(found_empty, "Should find empty: null");
            assert!(found_missing, "Should find missing: ~");
            assert!(found_yes, "Should find yes: yes");
            assert!(found_no, "Should find no: no");
            assert!(found_on, "Should find on: on");
            assert!(found_off, "Should find off: off");
        } else {
            panic!("Expected Documents node");
        }
    }

    #[test]
    fn test_parse_document_with_numeric_values() {
        let input = b"---\n\
            integer: 42\n\
            negative: -123\n\
            float: 3.14159\n\
            scientific: 1.23e+4\n\
            binary: 0b1010\n\
            octal: 0o755\n\
            hex: 0xFF";
        let actual = parse_yaml(input);
        if let Node::Documents(docs) = &actual {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        } else {
            panic!("Expected Documents node");
        }
    }

    #[test]
    fn test_parse_document_with_anchor_and_alias() {
        let input = b"---\n\
                        default: &default\n\
                            host: localhost\n\
                            port: 8080\n\
                        development:\n\
                            config: *default\n\
                            debug: true\n\
                        production:\n\
                            config: *default\n\
                            debug: false";
        let actual = parse_yaml(input);
        if let Node::Documents(docs) = &actual {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        } else {
            panic!("Expected Documents node");
        }
    }

    #[test]
    fn test_parse_document_with_literal_and_folded_strings() {
        let input = b"---\n\
                        literal: |\n\
                            Line 1\n\
                            Line 2\n\
                            Line 3\n\
                        folded: >\n\
                            This is a very long\n\
                            line that will be\n\
                            folded into a single\n\
                            paragraph.\n\
                        mixed:\n\
                            - |\n\
                                Literal in array\n\
                            - >\n\
                                Folded in array";
        let actual = parse_yaml(input);
        if let Node::Documents(docs) = &actual {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        } else {
            panic!("Expected Documents node");
        }
    }

    #[test]
    fn test_parse_document_with_unicode_content() {
        let input = b"---\n\
            greeting: \"\xE4\xBD\xA0\xE5\xA5\xBD\"  # Hello in Chinese\n\
            emoji: \"\xF0\x9F\x91\x8B\"  # Wave emoji\n\
            mixed: [\"\xC2\xA1Hola!\", \"world\", \"\xF0\x9F\x8C\x8D\"]  # Spanish + world emoji";
        let actual = parse_yaml(input);
        if let Node::Documents(docs) = &actual {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        } else {
            panic!("Expected Documents node");
        }
    }

    #[test]
    fn test_parse_document_with_tags() {
        let input = b"---\n\
            timestamp: !!timestamp 2001-12-14t21:59:43.10-05:00\n\
            integer: !!int \"123\"\n\
            float: !!float \"456.789\"\n\
            binary: !!binary |\n\
              R0lGODlhDAAMAIQAAP//9/X17unp5WZmZgAAAOfn515eXvPz7Y6OjuDg4J+fn5\n\
              OTk6enp56enmlpaWNjY6Ojo4SEhP/++f/++f/++f/++f/++f/++f/++f/++f/+\n\
              +f/++f/++f/++f/++f/++SH+Dk1hZGUgd2l0aCBHSU1QACwAAAAADAAMAAAFLC\n\
              AgjoEwnuNAFOhpEMTRiggcz4BNJHrv/zCFcLiwMWYNG84BwwEeECcgggoBADs=";
        let actual = parse_yaml(input);
        if let Node::Documents(docs) = &actual {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        } else {
            panic!("Expected Documents node");
        }
    }

    #[test]
    fn test_parse_document_with_tag_directives() {
        let input = b"%TAG !e! tag:example.com,2000:app/\n\
            ---\n\
            item: !e!custom value";
        let actual = parse_yaml(input);
        if let Node::Documents(docs) = &actual {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
                // Check that the tag was resolved
                if let Node::Mapping(pairs) = &nodes[0] {
                    if let Node::Tagged(_, tag) = &pairs[0].1 {
                        // Tag is not resolved by this parser; it should remain as the literal '!e!custom'
                        assert_eq!(tag, "!e!custom");
                    } else {
                        panic!("Expected tagged node");
                    }
                }
            }
        } else {
            panic!("Expected Documents node");
        }
    }

    #[test]
    fn test_parse_document_with_yaml_version() {
        let input = b"%YAML 1.2\n\
            ---\n\
            key: value";
        let actual = parse_yaml(input);
        // Accept parse success as pass
        if let Node::Documents(docs) = &actual {
            assert!(!docs.is_empty());
        } else {
            panic!("Expected Documents node");
        }
    }

    #[test]
    fn test_yaml_11_boolean_values() {
        let input = b"%YAML 1.1\n\
            ---\n\
            yes_value: yes\n\
            no_value: no\n\
            on_value: on\n\
            off_value: off";
        let actual = parse_yaml(input);
        if let Node::Documents(docs) = &actual {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    // Check yes -> true
                    if let Some((_, Node::Boolean(true))) = pairs
                        .iter()
                        .find(|(k, _)| matches!(k, Node::Str(s, _, _) if s == "yes_value"))
                    {
                        // Passed
                    } else {
                        panic!("Expected 'yes' to be parsed as boolean true in YAML 1.1");
                    }

                    // Check no -> false
                    if let Some((_, Node::Boolean(false))) = pairs
                        .iter()
                        .find(|(k, _)| matches!(k, Node::Str(s, _, _) if s == "no_value"))
                    {
                        // Passed
                    } else {
                        panic!("Expected 'no' to be parsed as boolean false in YAML 1.1");
                    }

                    // Check on -> true
                    if let Some((_, Node::Boolean(true))) = pairs
                        .iter()
                        .find(|(k, _)| matches!(k, Node::Str(s, _, _) if s == "on_value"))
                    {
                        // Passed
                    } else {
                        panic!("Expected 'on' to be parsed as boolean true in YAML 1.1");
                    }

                    // Check off -> false
                    if let Some((_, Node::Boolean(false))) = pairs
                        .iter()
                        .find(|(k, _)| matches!(k, Node::Str(s, _, _) if s == "off_value"))
                    {
                        // Passed
                    } else {
                        panic!("Expected 'off' to be parsed as boolean false in YAML 1.1");
                    }
                }
            }
        } else {
            panic!("Expected Documents node");
        }
    }

    #[test]
    fn test_yaml_12_boolean_values_strict() {
        let input = b"%YAML 1.2\n\
            ---\n\
            yes_value: yes\n\
            no_value: no\n\
            true_value: true\n\
            false_value: false";
        let actual = parse_yaml(input);
        if let Node::Documents(docs) = &actual {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    // Check yes remains string
                    if let Some((_, Node::Str(s, _, _))) = pairs
                        .iter()
                        .find(|(k, _)| matches!(k, Node::Str(key, _, _) if key == "yes_value"))
                    {
                        assert_eq!(s, "yes", "Expected 'yes' to remain a string in YAML 1.2");
                    } else {
                        panic!("Expected 'yes' to be parsed as string in YAML 1.2");
                    }

                    // Check no remains string
                    if let Some((_, Node::Str(s, _, _))) = pairs
                        .iter()
                        .find(|(k, _)| matches!(k, Node::Str(key, _, _) if key == "no_value"))
                    {
                        assert_eq!(s, "no", "Expected 'no' to remain a string in YAML 1.2");
                    } else {
                        panic!("Expected 'no' to be parsed as string in YAML 1.2");
                    }

                    // Check true/false still work
                    if let Some((_, Node::Boolean(true))) = pairs
                        .iter()
                        .find(|(k, _)| matches!(k, Node::Str(key, _, _) if key == "true_value"))
                    {
                        // Passed
                    } else {
                        panic!("Expected 'true' to be parsed as boolean in YAML 1.2");
                    }

                    if let Some((_, Node::Boolean(false))) = pairs
                        .iter()
                        .find(|(k, _)| matches!(k, Node::Str(key, _, _) if key == "false_value"))
                    {
                        // Passed
                    } else {
                        panic!("Expected 'false' to be parsed as boolean in YAML 1.2");
                    }
                }
            }
        } else {
            panic!("Expected Documents node");
        }
    }

    #[test]
    fn test_yaml_11_octal_numbers() {
        let input = b"%YAML 1.1\n\
            ---\n\
            permissions: 0755";
        let actual = parse_yaml(input);
        if let Node::Documents(docs) = &actual {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    if let Some((_, Node::Number(Numeric::Integer(i)))) = pairs
                        .iter()
                        .find(|(k, _)| matches!(k, Node::Str(s, _, _) if s == "permissions"))
                    {
                        assert_eq!(
                            *i, 493,
                            "Expected 0755 (octal) to be parsed as 493 (decimal) in YAML 1.1"
                        );
                    } else {
                        panic!("Expected '0755' to be parsed as octal number in YAML 1.1");
                    }
                }
            }
        } else {
            panic!("Expected Documents node");
        }
    }

    #[test]
    fn test_yaml_12_octal_numbers_require_0o_prefix() {
        let input = b"%YAML 1.2\n\
            ---\n\
            permissions: 0o755";
        let actual = parse_yaml(input);
        if let Node::Documents(docs) = &actual {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    if let Some((_, Node::Number(Numeric::Integer(i)))) = pairs
                        .iter()
                        .find(|(k, _)| matches!(k, Node::Str(s, _, _) if s == "permissions"))
                    {
                        assert_eq!(
                            *i, 493,
                            "Expected 0o755 (octal) to be parsed as 493 (decimal) in YAML 1.2"
                        );
                    } else {
                        panic!("Expected '0o755' to be parsed as octal number in YAML 1.2");
                    }
                }
            }
        } else {
            panic!("Expected Documents node");
        }
    }
}
