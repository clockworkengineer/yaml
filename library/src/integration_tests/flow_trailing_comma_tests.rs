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
}
