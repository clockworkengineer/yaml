#[cfg(test)]
mod test_flow_sequence_as_key {
    use crate::{BufferSource, parse};

    #[test]
    fn test_sbg9_flow_sequence_as_mapping_key() {
        // Test: Flow sequence as a mapping key (valid YAML)
        let yaml = b"{a: [b, c], [d, e]: f}";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        if let Err(ref e) = result {
            println!("Error: {}", e);
        }
        if let Ok(ref res) = result {
            println!("Parsed: {:?}", res);
        }
        assert!(result.is_ok(), "Should accept flow sequence as mapping key");
    }

    #[test]
    fn test_sbg9_exact_input() {
        // Test with exact SBG9 input from test suite
        let yaml = b"{a: [b, c], [d, e]: f}\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        if let Err(ref e) = result {
            println!("Error parsing SBG9: {}", e);
        }
        if let Ok(ref res) = result {
            println!("Parsed SBG9: {:?}", res);
        }
        assert!(result.is_ok(), "Should parse SBG9 successfully");
    }
}
