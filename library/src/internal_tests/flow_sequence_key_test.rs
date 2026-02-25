// =====================================================================================
//  File: flow_sequence_key_test.rs
//  Location: library/src/internal_tests/
// -------------------------------------------------------------------------------------
//  Purpose:
//      Internal tests for YAML flow sequence usage as mapping keys in the yaml_lib crate.
//      These tests validate correct parsing and handling of flow sequences as keys in mappings,
//      ensuring compliance with the YAML specification and support for advanced key types.
//
//  Context:
//      - Part of the yaml_lib project, a Rust YAML parser/serializer.
//      - Focuses on edge cases for mapping keys and flow sequence syntax.
//      - Ensures robust handling of non-scalar mapping keys in YAML documents.
//
// -------------------------------------------------------------------------------------
//  Test Coverage:
//      - Flow sequences as mapping keys
//      - Compliance with YAML mapping key rules
//      - Edge cases for advanced key types
// =====================================================================================

#[cfg(test)]
mod tests {
    use crate::{BufferSource, parse};

    #[test]
    fn test_sbg9_flow_sequence_as_mapping_key() {
        // Test: Flow sequence as a mapping key (valid YAML)
        let yaml = b"{a: [b, c], [d, e]: f}";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        if let Err(ref _e) = result {
            #[cfg(feature = "debug-trace")]
            println!("Error: {}", _e);
        }
        if let Ok(ref _res) = result {
            #[cfg(feature = "debug-trace")]
            println!("Parsed: {:?}", _res);
        }
        assert!(result.is_ok(), "Should accept flow sequence as mapping key");
    }

    #[test]
    fn test_sbg9_exact_input() {
        // Test with exact SBG9 input from test suite
        let yaml = b"{a: [b, c], [d, e]: f}\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        if let Err(ref _e) = result {
            #[cfg(feature = "debug-trace")]
            println!("Error parsing SBG9: {}", _e);
        }
        if let Ok(ref _res) = result {
            #[cfg(feature = "debug-trace")]
            println!("Parsed SBG9: {:?}", _res);
        }
        assert!(result.is_ok(), "Should parse SBG9 successfully");
    }
}
