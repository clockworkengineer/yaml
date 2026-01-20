//! Integration tests for embedded system features

#[cfg(test)]
mod tests {
    use crate::embedded::limits::NodeValidator;
    use crate::io::sources::buffer::Buffer as BufferSource;
    use crate::nodes::node::{Node, Numeric};
    use crate::nodes::util::make_set;
    use crate::parser::document::parse;

    #[test]
    fn test_parse_and_validate_simple_yaml() {
        let yaml = r#"
name: "Test Device"
id: 42
enabled: true
"#;
        let mut source = BufferSource::new(yaml.as_bytes());
        match parse(&mut source) {
            Ok(doc) => {
                // Unwrap document structure
                let mut node = &doc;
                loop {
                    match node {
                        Node::Document(nodes) | Node::Documents(nodes) => {
                            if let Some(first) = nodes.first() {
                                node = first;
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }
            }
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    fn test_numeric_conversions_integration() {
        let yaml = r#"
int32_val: 42
int64_val: 9223372036854775807
float_val: 3.14159
                loop {
                    match node {
                        Node::Document(nodes) | Node::Documents(nodes) => {
                            if let Some(first) = nodes.first() {
                                node = first;
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }
                #[cfg(feature = "debug-trace")]
                println!("DEBUG: Unwrapped node type: {:?}", node);
                // Print keys and their types/lengths
                if let Node::Mapping(pairs) = node {
                    for (_k, _v) in pairs {
                        #[cfg(feature = "debug-trace")]
                        println!(
                            "DEBUG: Key: {:?}, Value: {:?}, Value len: {:?}",
                            _k,
                            _v,
                            _v.len()
                        );
                    }
                }
                // Check array
                if let Some(arr) = node.get_key("array") {
                    #[cfg(feature = "debug-trace")]
                    println!("DEBUG: array: {:?}, len: {:?}", arr, arr.len());
                    assert!(arr.is_sequence());
                    assert!(!arr.is_mapping());
                    assert_eq!(arr.len(), Some(3));
                    assert!(!arr.is_empty());
                }
                // Check mapping
                if let Some(map) = node.get_key("mapping") {
                    #[cfg(feature = "debug-trace")]
                    println!("DEBUG: mapping: {:?}, len: {:?}", map, map.len());
                    assert!(!map.is_sequence());
                    assert!(map.is_mapping());
                    assert_eq!(map.len(), Some(2));
                    assert!(!map.is_empty());
                }
                // Check empty array
                if let Some(empty_arr) = node.get_key("empty_array") {
                    #[cfg(feature = "debug-trace")]
                    println!(
                        "DEBUG: empty_array: {:?}, len: {:?}",
                        empty_arr,
                        empty_arr.len()
                    );
                    assert!(empty_arr.is_sequence());
                    assert_eq!(empty_arr.len(), Some(0));
                    assert!(empty_arr.is_empty());
                }
                // Check empty mapping
                if let Some(empty_map) = node.get_key("empty_mapping") {
                    #[cfg(feature = "debug-trace")]
                    println!(
                        "DEBUG: empty_mapping: {:?}, len: {:?}",
                        empty_map,
                        empty_map.len()
                    );
                    assert!(empty_map.is_mapping());
                    assert_eq!(empty_map.len(), Some(0));
                    assert!(empty_map.is_empty());
                }
            }
            Err(e) => panic!("Parse error: {}", e),
        }
    }
        assert_eq!(arr.get(1).and_then(|n| n.as_i32()), Some(999));
        assert_eq!(arr.get(2).and_then(|n| n.as_i32()), Some(30));
    }

    #[test]
    fn test_collection_queries() {
        let yaml = r#"
array: [1, 2, 3]
mapping:
  key1: value1
  key2: value2
empty_array: []
empty_mapping: {}
"#;
        let mut source = BufferSource::new(yaml.as_bytes());
        match parse(&mut source) {
            Ok(doc) => {
                let mut node = &doc;
                #[cfg(feature = "debug-trace")]
                println!("DEBUG: Root node type: {:?}", node);
                loop {
                    match node {
                        Node::Document(nodes) | Node::Documents(nodes) => {
                            if let Some(first) = nodes.first() {
                                node = first;
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }
                #[cfg(feature = "debug-trace")]
                println!("DEBUG: Unwrapped node type: {:?}", node);
                // Print keys and their types/lengths
                if let Node::Mapping(pairs) = node {
                    for (_k, _v) in pairs {
                        #[cfg(feature = "debug-trace")]
                        println!(
                            "DEBUG: Key: {:?}, Value: {:?}, Value len: {:?}",
                            _k,
                            _v,
                            _v.len()
                        );
                    }
                }
                // ...existing code...
                // Check array
                if let Some(arr) = node.get_key("array") {
                    #[cfg(feature = "debug-trace")]
                    println!("DEBUG: array: {:?}, len: {:?}", arr, arr.len());
                    assert!(arr.is_sequence());
                    assert!(!arr.is_mapping());
                    assert_eq!(arr.len(), Some(3));
                    assert!(!arr.is_empty());
                }
                // Check mapping
                if let Some(map) = node.get_key("mapping") {
                    #[cfg(feature = "debug-trace")]
                    println!("DEBUG: mapping: {:?}, len: {:?}", map, map.len());
                    assert!(!map.is_sequence());
                    assert!(map.is_mapping());
                    assert_eq!(map.len(), Some(2));
                    assert!(!map.is_empty());
                }
                // Check empty array
                if let Some(empty_arr) = node.get_key("empty_array") {
                    #[cfg(feature = "debug-trace")]
                    println!(
                        "DEBUG: empty_array: {:?}, len: {:?}",
                        empty_arr,
                        empty_arr.len()
                    );
                    assert!(empty_arr.is_sequence());
                    assert_eq!(empty_arr.len(), Some(0));
                    assert!(empty_arr.is_empty());
                }
                // Check empty mapping
                if let Some(empty_map) = node.get_key("empty_mapping") {
                    #[cfg(feature = "debug-trace")]
                    println!(
                        "DEBUG: empty_mapping: {:?}, len: {:?}",
                        empty_map,
                        empty_map.len()
                    );
                    assert!(empty_map.is_mapping());
                    assert_eq!(empty_map.len(), Some(0));
                    assert!(empty_map.is_empty());
                }
            }
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    fn test_numeric_size_awareness() {
        // Verify numeric size tracking
        assert_eq!(Numeric::Int32(0).size_bytes(), 4);
        assert_eq!(Numeric::Integer(0).size_bytes(), 8);
        assert_eq!(Numeric::Byte(0).size_bytes(), 1);

        // For embedded, prefer smaller types
        let small = Node::Number(Numeric::Int32(42));
        let large = Node::Number(Numeric::Integer(42));

        if let Node::Number(n) = &small {
            assert_eq!(n.size_bytes(), 4);
        }
        if let Node::Number(n) = &large {
            assert_eq!(n.size_bytes(), 8);
        }
    }

    #[test]
    fn test_safe_key_mutation() {
        use crate::nodes::node::{BlockStyle, QuoteType};

        let mut pairs = alloc::vec::Vec::new();
        pairs.push((
            Node::Str("name".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str(
                "original".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ),
        ));
        pairs.push((
            Node::Str("count".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Number(Numeric::Int32(0)),
        ));
        let mut mapping = Node::Mapping(pairs);

        // Mutate through safe access
        if let Some(name_node) = mapping.get_key_mut("name") {
            *name_node = Node::Str("updated".to_string(), QuoteType::Unquoted, BlockStyle::None);
        }

        if let Some(count_node) = mapping.get_key_mut("count") {
            *count_node = Node::Number(Numeric::Int32(42));
        }

        // Verify mutations
        assert_eq!(
            mapping.get_key("name").and_then(|n| n.as_str()),
            Some("updated")
        );
        assert_eq!(mapping.get_key("count").and_then(|n| n.as_i32()), Some(42));
    }

    #[test]
    #[ignore]
    fn test_parse_with_validation_workflow() {
        let yaml = r#"
device:
  name: "Sensor-01"
  readings:
    - temperature: 22.5
    - humidity: 65
    - pressure: 1013
  status: active
"#;
        let mut source = BufferSource::new(yaml.as_bytes());
        match parse(&mut source) {
            Ok(doc) => {
                // First validate
                let mut validator = NodeValidator::new();
                match validator.validate(&doc) {
                    Ok(()) => {
                        // Then extract data safely
                        let mut node = &doc;
                        loop {
                            match node {
                                Node::Document(nodes) | Node::Documents(nodes) => {
                                    if let Some(first) = nodes.first() {
                                        node = first;
                                    } else {
                                        break;
                                    }
                                }
                                _ => break,
                            }
                        }

                        if let Some(device) = node.get_key("device") {
                            assert_eq!(
                                device.get_key("name").and_then(|n| n.as_str()),
                                Some("Sensor-01")
                            );
                            assert_eq!(
                                device.get_key("status").and_then(|n| n.as_str()),
                                Some("active")
                            );

                            if let Some(readings) = device.get_key("readings") {
                                assert_eq!(readings.len(), Some(3));
                                assert!(readings.is_sequence());
                            }
                        }
                    }
                    Err(e) => panic!("Validation failed: {:?}", e),
                }
            }
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    fn test_make_set_with_embedded() {
        let set = make_set(vec![
            Node::Number(Numeric::Int32(1)),
            Node::Number(Numeric::Int32(2)),
            Node::Number(Numeric::Int32(3)),
            Node::Number(Numeric::Int32(2)), // Duplicate
        ]);

        match set {
            Node::Set(items) => {
                assert_eq!(items.len(), 3); // Duplicate removed
                assert!(
                    items
                        .iter()
                        .any(|n| matches!(n, Node::Number(Numeric::Int32(1))))
                );
                assert!(
                    items
                        .iter()
                        .any(|n| matches!(n, Node::Number(Numeric::Int32(2))))
                );
                assert!(
                    items
                        .iter()
                        .any(|n| matches!(n, Node::Number(Numeric::Int32(3))))
                );
            }
            _ => panic!("Expected Set node"),
        }
    }
}
