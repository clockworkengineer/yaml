#[cfg(test)]
mod test_flow_mapping_linebreak {
    use crate::{BufferSource, parse};
    use crate::nodes::node::{Node, Document};

    #[test]
    fn test_5mud_flow_mapping_linebreak() {
        // Flow mapping with a newline between key and colon
        let yaml = b"---\n{ \"foo\"\n  :bar }";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_ok(), "Parser should accept 5MUD case: {:?}", result);
        let root = result.unwrap();
        if let Node::Documents(docs) = root {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    let (k, v) = &pairs[0];
                    if let Node::Str(s, _, _) = k { assert_eq!(s, "foo"); } else { panic!("key not Str: {:?}", k); }
                    if let Node::Str(s, _, _) = v { assert_eq!(s, ":bar"); } else { panic!("value not Str: {:?}", v); }
                    return;
                }
            }
        }
        panic!("Unexpected AST shape for 5MUD");
    }
}
