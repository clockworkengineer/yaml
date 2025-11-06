///
/// Tag and coercion tests: YAML tags (!!int, !!float, !!str, !!timestamp, etc.) and type coercion.
///
#[cfg(test)]
mod tests {
    use crate::nodes::node::QuoteType;
    use crate::{BufferDestination, BufferSource, Node, Node::Document, Numeric, parse, stringify};

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
                        assert_eq!(s.as_str(), "123");
                        assert_eq!(*qt, QuoteType::Unquoted);
                        return;
                    }
                }
            }
        }
        panic!("Expected tagged string node not found");
    }

    #[test]
    fn test_stringify_preserves_tag_token() {
        let mut source = BufferSource::new(b"value: !tag 123");
        let node = parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        let out = dest.to_string();
        assert!(
            out.contains("!tag"),
            "stringify should preserve tag: {}",
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
            "stringify should preserve custom tag: {}",
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
                            } else if ks == "b" {
                                found_b = Some(v.clone());
                            }
                        }
                    }

                    // Both should resolve to the same tagged string value
                    assert!(found_a.is_some(), "Key 'a' not found");
                    assert!(found_b.is_some(), "Key 'b' not found");

                    // Verify the tag was applied correctly
                    if let Some(Node::Anchored(anchored_node, _)) = found_a {
                        if let Node::Str(s, _, _) = anchored_node.as_ref() {
                            assert_eq!(s, "123");
                        } else {
                            panic!("Expected tagged string in anchor");
                        }
                    } else if let Some(Node::Str(s, _, _)) = found_a {
                        assert_eq!(s, "123");
                    } else {
                        panic!("Expected anchored or string node for 'a'");
                    }
                    return;
                }
            }
        }
        panic!("Expected tagged anchor and alias structure not found");
    }
}
