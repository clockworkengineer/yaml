#[cfg(test)]
mod test_pw8x {
    use crate::io::sources::buffer::Buffer;
    use crate::nodes::node::Node;
    use crate::parser;
    use crate::parser::directives::DirectiveContext;

    #[test]
    fn test_mapping_with_anchor_on_empty_key() {
        // Simpler case: just the mapping part
        let yaml = b"&a : a\nb: &b\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        
        let result = crate::parser::document::mapping::parse_mapping(&mut source, 0, &directives);
        
        match result {
            Ok(Node::Mapping(pairs)) => {
                assert_eq!(pairs.len(), 2, "Should have 2 mapping pairs");
                eprintln!("SUCCESS: Got {} pairs", pairs.len());
            }
            Ok(other) => panic!("Expected Mapping, got {:?}", other),
            Err(e) => panic!("Parse failed: {}", e),
        }
    }

    #[test]
    fn test_sequence_with_nested_mapping() {
        // Full PW8X third item
        let yaml = b"-\n  &a : a\n  b: &b\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        
        let result = parser::parse(&mut source);
        
        match result {
            Ok(_docs) => {
                eprintln!("SUCCESS: Parsed PW8X pattern");
            }
            Err(e) => {
                eprintln!("FAILED: {}", e);
                panic!("Parse failed: {}", e);
            }
        }
    }
}
