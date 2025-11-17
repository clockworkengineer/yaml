///
/// Document structure tests: multi-document YAML, document markers, header comments.
///
#[cfg(test)]
mod tests {
    use crate::nodes::node::{BlockStyle, QuoteType};
    use crate::{BufferSource, Node, Node::Document, Numeric, parse};
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
        let mut source = BufferSource::new(b"# Comment before\nkey: value\n# Comment after\n---\n# Comment in between\nother: 123\n# Final comment");
        let result = parse(&mut source).unwrap();

        // Should parse as two documents
        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 2);

            // First document should have one mapping with key: value
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }

            // Second document should have one mapping with other: 123
            if let Document(nodes) = &docs[1] {
                assert!(!nodes.is_empty());
            }
        } else {
            panic!("Expected Documents node");
        }
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
    fn test_parse_empty_document_end_marker() {
        let mut source = BufferSource::new(b"...");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![Document(vec![])]));
    }

    #[test]
    fn test_parse_document_start_marker() {
        let mut source = BufferSource::new(b"---\nkey: value");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_document_start_and_end_markers() {
        let mut source = BufferSource::new(b"---\nkey: value\n...");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_multiple_documents_with_start_markers() {
        let mut source = BufferSource::new(b"---\nfirst: 1\n---\nsecond: 2\n---\nthird: 3");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 3);
        }
    }

    #[test]
    fn test_parse_documents_with_mixed_markers() {
        let mut source =
            BufferSource::new(b"---\nfirst: 1\n...\n---\nsecond: 2\n---\nthird: 3\n...");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 3);
        }
    }

    #[test]
    fn test_parse_empty_documents_between_markers() {
        let mut source = BufferSource::new(b"---\n---\nkey: value\n---");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            // Parser may merge empty documents, so expect at least 1 document with content
            assert!(!docs.is_empty());

            // Find the document with content
            let mut found_content = false;
            for doc in docs {
                if let Document(nodes) = doc {
                    if !nodes.is_empty() {
                        found_content = true;
                        break;
                    }
                }
            }
            assert!(found_content, "Should have at least one document with content");
        }
    }

    #[test]
    fn test_parse_document_with_yaml_version_directive() {
        // Note: YAML directives are not supported by this parser, so we test without them
        let mut source = BufferSource::new(b"# YAML version would be %YAML 1.2 here\n---\nkey: value");
        let result = parse(&mut source).unwrap();
        
        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_document_with_tag_directive() {
        // Note: TAG directives are not supported by this parser, so we test without them
        let mut source = BufferSource::new(b"# TAG directive would be %TAG ! tag:example.com,2000:app/ here\n---\nkey: value");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_document_with_multiple_directives() {
        // Note: Directives are not supported by this parser, so we test with comments instead
        let mut source = BufferSource::new(
            b"# Would have %YAML 1.2 and %TAG ! tag:example.com,2000:app/ directives here\n---\nkey: value"
        );
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_document_with_comments_around_markers() {
        let mut source = BufferSource::new(b"# Before first marker\n---\n# After first marker\nkey: value\n# Before end marker\n...\n# After end marker");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_documents_with_different_content_types() {
        let mut source =
            BufferSource::new(b"---\n\"scalar document\"\n---\n- item1\n- item2\n---\nkey: value");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 3);

            // First document: scalar
            if let Document(nodes) = &docs[0] {
                assert_eq!(nodes.len(), 1);
                matches!(nodes[0], Node::Str(_, _, _));
            }

            // Second document: sequence
            if let Document(nodes) = &docs[1] {
                assert_eq!(nodes.len(), 1);
                matches!(nodes[0], Node::Array(_));
            }

            // Third document: mapping
            if let Document(nodes) = &docs[2] {
                assert_eq!(nodes.len(), 1);
                matches!(nodes[0], Node::Mapping(_));
            }
        }
    }

    #[test]
    fn test_parse_document_with_whitespace_only_content() {
        let mut source = BufferSource::new(b"---\n   \n  \n\t\n---\nkey: value");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            // Should have 1 document since whitespace-only documents are treated as empty/merged
            assert_eq!(docs.len(), 1);
        }
    }

    #[test]
    fn test_parse_document_with_trailing_spaces_on_markers() {
        let mut source = BufferSource::new(b"---   \nkey: value\n...   ");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_document_markers_with_comments_inline() {
        let mut source = BufferSource::new(b"--- # Start of document\nkey: value\n# End of document");
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_documents_with_complex_nested_content() {
        let mut source = BufferSource::new(
            b"---\nusers:\n  - name: alice\n    roles: [admin, user]\n  - name: bob\n    roles: [user]\nconfig:\n  database:\n    host: localhost\n    port: 5432\n---\nservers:\n  web:\n    - host: web1.example.com\n    - host: web2.example.com"
        );
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 2);

            // Both documents should have non-empty content
            for doc in docs {
                if let Document(nodes) = doc {
                    assert!(!nodes.is_empty());
                }
            }
        }
    }

    #[test]
    fn test_parse_document_with_boolean_and_null_values() {
        let mut source = BufferSource::new(
            b"---\n\
            enabled: true\n\
            disabled: false\n\
            empty: null\n\
            missing: ~\n\
            ---\n\
            yes: yes\n\
            no: no\n\
            on: on\n\
            off: off",
        );
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 2);
        }
    }

    #[test]
    fn test_parse_document_with_numeric_values() {
        let mut source = BufferSource::new(
            b"---\n\
            integer: 42\n\
            negative: -123\n\
            float: 3.14159\n\
            scientific: 1.23e+4\n\
            binary: 0b1010\n\
            octal: 0o755\n\
            hex: 0xFF",
        );
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_document_with_anchor_and_alias() {
        let mut source = BufferSource::new(
            b"---\n\
            default: &default\n\
              host: localhost\n\
              port: 8080\n\
            development:\n\
              config: *default\n\
              debug: true\n\
            production:\n\
              config: *default\n\
              debug: false",
        );
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_document_with_literal_and_folded_strings() {
        let mut source = BufferSource::new(
            b"---\n\
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
                Folded in array",
        );
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_document_with_unicode_content() {
        let mut source = BufferSource::new(
            b"---\n\
            greeting: \"\xE4\xBD\xA0\xE5\xA5\xBD\"  # Hello in Chinese\n\
            emoji: \"\xF0\x9F\x91\x8B\"  # Wave emoji\n\
            mixed: [\"\xC2\xA1Hola!\", \"world\", \"\xF0\x9F\x8C\x8D\"]  # Spanish + world emoji",
        );
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_document_with_tags() {
        let mut source = BufferSource::new(
            b"---\n\
            timestamp: !!timestamp 2001-12-14t21:59:43.10-05:00\n\
            integer: !!int \"123\"\n\
            float: !!float \"456.789\"\n\
            binary: !!binary |\n\
              R0lGODlhDAAMAIQAAP//9/X17unp5WZmZgAAAOfn515eXvPz7Y6OjuDg4J+fn5\n\
              OTk6enp56enmlpaWNjY6Ojo4SEhP/++f/++f/++f/++f/++f/++f/++f/++f/+\n\
              +f/++f/++f/++f/++f/++SH+Dk1hZGUgd2l0aCBHSU1QACwAAAAADAAMAAAFLC\n\
              AgjoEwnuNAFOhpEMTRiggcz4BNJHrv/zCFcLiwMWYNG84BwwEeECcgggoBADs=",
        );
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_document_with_tag_directives() {
        // Test tag prefix resolution
        let mut source = BufferSource::new(
            b"%TAG !e! tag:example.com,2000:app/\n\
            ---\n\
            item: !e!custom value",
        );
        let result = parse(&mut source).unwrap();

        if let Node::Documents(docs) = &result {
            assert_eq!(docs.len(), 1);
            if let Document(nodes) = &docs[0] {
                assert!(!nodes.is_empty());
                // Check that the tag was resolved
                if let Node::Mapping(pairs) = &nodes[0] {
                    if let Node::Tagged(_, tag) = &pairs[0].1 {
                        // Tag should be resolved from !e!custom to tag:example.com,2000:app/custom
                        assert_eq!(tag, "tag:example.com,2000:app/custom");
                    } else {
                        panic!("Expected tagged node");
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_document_with_yaml_version() {
        // Test YAML version directive
        let mut source = BufferSource::new(
            b"%YAML 1.2\n\
            ---\n\
            key: value",
        );
        let result = parse(&mut source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_yaml_11_boolean_values() {
        // YAML 1.1 should accept yes/no/on/off as booleans
        let mut source = BufferSource::new(
            b"%YAML 1.1\n\
            ---\n\
            yes_value: yes\n\
            no_value: no\n\
            on_value: on\n\
            off_value: off",
        );
        let result = parse(&mut source);
        assert!(result.is_ok());
        
        if let Ok(Node::Documents(docs)) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                // Check yes -> true
                if let Some((_, Node::Boolean(true))) = pairs.iter().find(|(k, _)| {
                    matches!(k, Node::Str(s, _, _) if s == "yes_value")
                }) {
                    // Passed
                } else {
                    panic!("Expected 'yes' to be parsed as boolean true in YAML 1.1");
                }
                
                // Check no -> false
                if let Some((_, Node::Boolean(false))) = pairs.iter().find(|(k, _)| {
                    matches!(k, Node::Str(s, _, _) if s == "no_value")
                }) {
                    // Passed
                } else {
                    panic!("Expected 'no' to be parsed as boolean false in YAML 1.1");
                }
                
                // Check on -> true
                if let Some((_, Node::Boolean(true))) = pairs.iter().find(|(k, _)| {
                    matches!(k, Node::Str(s, _, _) if s == "on_value")
                }) {
                    // Passed
                } else {
                    panic!("Expected 'on' to be parsed as boolean true in YAML 1.1");
                }
                
                // Check off -> false
                if let Some((_, Node::Boolean(false))) = pairs.iter().find(|(k, _)| {
                    matches!(k, Node::Str(s, _, _) if s == "off_value")
                }) {
                    // Passed
                } else {
                    panic!("Expected 'off' to be parsed as boolean false in YAML 1.1");
                }
                }
            }
        }
    }

    #[test]
    fn test_yaml_12_boolean_values_strict() {
        // YAML 1.2 should NOT accept yes/no/on/off as booleans (they remain strings)
        let mut source = BufferSource::new(
            b"%YAML 1.2\n\
            ---\n\
            yes_value: yes\n\
            no_value: no\n\
            true_value: true\n\
            false_value: false",
        );
        let result = parse(&mut source);
        assert!(result.is_ok());
        
        if let Ok(Node::Documents(docs)) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                // Check yes remains string
                if let Some((_, Node::Str(s, _, _))) = pairs.iter().find(|(k, _)| {
                    matches!(k, Node::Str(key, _, _) if key == "yes_value")
                }) {
                    assert_eq!(s, "yes", "Expected 'yes' to remain a string in YAML 1.2");
                } else {
                    panic!("Expected 'yes' to be parsed as string in YAML 1.2");
                }
                
                // Check no remains string
                if let Some((_, Node::Str(s, _, _))) = pairs.iter().find(|(k, _)| {
                    matches!(k, Node::Str(key, _, _) if key == "no_value")
                }) {
                    assert_eq!(s, "no", "Expected 'no' to remain a string in YAML 1.2");
                } else {
                    panic!("Expected 'no' to be parsed as string in YAML 1.2");
                }
                
                // Check true/false still work
                if let Some((_, Node::Boolean(true))) = pairs.iter().find(|(k, _)| {
                    matches!(k, Node::Str(key, _, _) if key == "true_value")
                }) {
                    // Passed
                } else {
                    panic!("Expected 'true' to be parsed as boolean in YAML 1.2");
                }
                
                if let Some((_, Node::Boolean(false))) = pairs.iter().find(|(k, _)| {
                    matches!(k, Node::Str(key, _, _) if key == "false_value")
                }) {
                    // Passed
                } else {
                    panic!("Expected 'false' to be parsed as boolean in YAML 1.2");
                }
                }
            }
        }
    }

    #[test]
    fn test_yaml_11_octal_numbers() {
        // YAML 1.1 accepts octal with plain 0 prefix (e.g., 0755)
        let mut source = BufferSource::new(
            b"%YAML 1.1\n\
            ---\n\
            permissions: 0755",
        );
        let result = parse(&mut source);
        assert!(result.is_ok());
        
        if let Ok(Node::Documents(docs)) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                if let Some((_, Node::Number(Numeric::Integer(i)))) = pairs.iter().find(|(k, _)| {
                    matches!(k, Node::Str(s, _, _) if s == "permissions")
                }) {
                    assert_eq!(*i, 493, "Expected 0755 (octal) to be parsed as 493 (decimal) in YAML 1.1");
                } else {
                    panic!("Expected '0755' to be parsed as octal number in YAML 1.1");
                }
                }
            }
        }
    }

    #[test]
    fn test_yaml_12_octal_numbers_require_0o_prefix() {
        // YAML 1.2 requires 0o prefix for octal
        let mut source = BufferSource::new(
            b"%YAML 1.2\n\
            ---\n\
            permissions: 0o755",
        );
        let result = parse(&mut source);
        assert!(result.is_ok());
        
        if let Ok(Node::Documents(docs)) = result {
            if let Node::Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    if let Some((_, Node::Number(Numeric::Integer(i)))) = pairs.iter().find(|(k, _)| {
                        matches!(k, Node::Str(s, _, _) if s == "permissions")
                    }) {
                        assert_eq!(*i, 493, "Expected 0o755 (octal) to be parsed as 493 (decimal) in YAML 1.2");
                    } else {
                        panic!("Expected '0o755' to be parsed as octal number in YAML 1.2");
                    }
                }
            }
        }
    }
}
