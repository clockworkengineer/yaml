// =====================================================================================
//  File: flow_mapping_linebreak_tests.rs
//  Location: library/src/integration_tests/
// -------------------------------------------------------------------------------------
//  Purpose:
//      Integration tests for YAML flow mapping linebreak handling in the yaml_lib crate.
//      These tests validate correct parsing of flow mappings with line breaks between keys,
//      colons, and values, ensuring compliance with the YAML specification.
//
//  Context:
//      - Part of the yaml_lib project, a Rust YAML parser/serializer.
//      - Focuses on edge cases for flow mapping formatting and line breaks.
//      - Ensures robust handling of non-standard but valid YAML formatting.
//
// -------------------------------------------------------------------------------------
//  Test Coverage:
//      - Flow mappings with line breaks
//      - Key, colon, and value separation edge cases
//      - Compliance with YAML flow mapping rules
// =====================================================================================

#[cfg(test)]
mod tests {
    use crate::nodes::node::{Document, Node};
    use crate::{BufferSource, parse};

    #[test]
    fn test_5mud_flow_mapping_linebreak() {
        // Flow mapping with a newline between key and colon
        let yaml = b"---\n{ \"foo\"\n  :bar }";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_ok(),
            "Parser should accept 5MUD case: {:?}",
            result
        );
        let root = result.unwrap();
        if let Node::Documents(docs) = root {
            if let Document(nodes) = &docs[0] {
                if let Node::Mapping(pairs) = &nodes[0] {
                    assert_eq!(pairs.len(), 1);
                    let (k, v) = &pairs[0];
                    if let Node::Str(s, _, _) = k {
                        assert_eq!(s, "foo");
                    } else {
                        panic!("key not Str: {:?}", k);
                    }
                    if let Node::Str(s, _, _) = v {
                        assert_eq!(s, ":bar");
                    } else {
                        panic!("value not Str: {:?}", v);
                    }
                    return;
                }
            }
        }
        panic!("Unexpected AST shape for 5MUD");
    }
}
