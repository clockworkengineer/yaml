#[cfg(test)]
mod test_flow_trailing_comma {
    use crate::{BufferSource, parse};

    #[test]
    fn test_flow_mapping_trailing_comma_simple() {
        let yaml = b"{ a: 1, b: 2, }";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_ok(), "Should parse flow mapping with trailing comma");
    }

    #[test]
    fn test_flow_sequence_trailing_comma_simple() {
        let yaml = b"[ 1, 2, 3, ]";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_ok(), "Should parse flow sequence with trailing comma");
    }

    #[test]
    fn test_5c5m_flow_mappings() {
        // Test 5C5M: Spec Example 7.15. Flow Mappings
        let yaml = b"- { one : two , three: four , }\n- {five: six,seven : eight}\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_ok(), "Should parse 5C5M test case");
    }

    #[test]
    fn test_5kje_flow_sequences() {
        // Test 5KJE: Spec Example 7.13. Flow Sequence
        let yaml = b"- [ one, two, ]\n- [three ,four]\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_ok(), "Should parse 5KJE test case");
    }

    #[test]
    fn test_block_sequence_with_flow_mapping_no_trailing() {
        // Block sequence with flow mapping WITHOUT trailing comma - should work
        let yaml = b"- {a: 1, b: 2}\n- {c: 3}";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_ok(), "Should parse block sequence with flow mappings");
    }

    #[test]
    fn test_block_sequence_with_flow_sequence_no_trailing() {
        // Block sequence with flow sequence WITHOUT trailing comma - should work
        let yaml = b"- [1, 2]\n- [3, 4]";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_ok(), "Should parse block sequence with flow sequences");
    }

    #[test]
    fn test_single_block_item_flow_mapping_trailing() {
        // Single block sequence item with flow mapping with trailing comma
        let yaml = b"- { a: 1, }";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_ok(), "Should parse single item");
    }

    #[test]
    fn test_two_block_items_flow_mapping_no_trailing() {
        // Two block sequence items, first has trailing comma
        let yaml = b"- { a: 1, }\n- { b: 2 }";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_ok(), "Should parse two items");
    }
}
