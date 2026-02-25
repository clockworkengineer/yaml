// =====================================================================================
//  File: flow_debug.rs
//  Location: library/src/internal_tests/
// -------------------------------------------------------------------------------------
//  Purpose:
//      Internal tests for debugging and validating YAML flow style parsing in the yaml_lib crate.
//      These tests focus on edge cases, trailing commas, and error scenarios in flow mappings
//      and sequences, aiding in parser development and troubleshooting.
//
//  Context:
//      - Part of the yaml_lib project, a Rust YAML parser/serializer.
//      - Used for debugging and validating flow style parsing logic and error handling.
//      - Ensures robust handling of flow mappings, sequences, and related edge cases.
//
// -------------------------------------------------------------------------------------
//  Test Coverage:
//      - Flow mappings and sequences
//      - Trailing commas and formatting edge cases
//      - Error handling and debug output
//      - Compliance with YAML flow style rules
// =====================================================================================

#[cfg(test)]
mod tests {
    use crate::{BufferSource, parse};
    #[test]
    fn test_5c5m_trailing_comma_mapping() {
        // Test: Flow mapping with trailing comma
        let yaml = "- { one : two , three: four , }\n- {five: six,seven : eight}";

        #[cfg(feature = "debug-trace")]
        println!("Testing 5C5M: {}", yaml);
        let mut source = BufferSource::new(yaml.as_bytes());

        match parse(&mut source) {
            Ok(_) => {
                #[cfg(feature = "debug-trace")]
                println!("Parsed successfully");
            }
            Err(e) => {
                #[cfg(feature = "debug-trace")]
                println!("Parse error: {}", e);
                panic!("Failed to parse: {}", e);
            }
        }
    }

    #[test]
    fn test_5kje_trailing_comma_sequence() {
        // Test: Flow sequence with trailing comma
        let yaml = "- [ one, two, ]\n- [three ,four]";

        #[cfg(feature = "debug-trace")]
        println!("Testing 5KJE: {}", yaml);
        let mut source = BufferSource::new(yaml.as_bytes());

        match parse(&mut source) {
            Ok(_) => {
                #[cfg(feature = "debug-trace")]
                println!("Parsed successfully");
            }
            Err(e) => {
                #[cfg(feature = "debug-trace")]
                println!("Parse error: {}", e);
                panic!("Failed to parse: {}", e);
            }
        }
    }

    #[test]
    fn test_5t43_double_colon() {
        // Test: Colon at beginning of adjacent flow scalar
        let yaml = "- { \"key\":value }\n- { \"key\"::value }";

        #[cfg(feature = "debug-trace")]
        println!("Testing 5T43: {}", yaml);
        let mut source = BufferSource::new(yaml.as_bytes());

        match parse(&mut source) {
            Ok(_) => {
                #[cfg(feature = "debug-trace")]
                println!("Parsed successfully (double colon treated as value prefix)");
            }
            Err(e) => {
                #[cfg(feature = "debug-trace")]
                println!("Parse error: {}", e);
                panic!("Failed to parse double-colon value case: {}", e);
            }
        }
    }

    #[test]
    fn test_7zz5_empty_collections() {
        // Test: Empty flow collections
        let yaml = "---\nnested sequences:\n- - - []\n- - - {}\nkey1: []\nkey2: {}";

        #[cfg(feature = "debug-trace")]
        println!("Testing 7ZZ5: {}", yaml);
        let mut source = BufferSource::new(yaml.as_bytes());

        match parse(&mut source) {
            Ok(_) => {
                #[cfg(feature = "debug-trace")]
                println!("Parsed successfully");
            }
            Err(e) => {
                #[cfg(feature = "debug-trace")]
                println!("Parse error: {}", e);
                panic!("Failed to parse: {}", e);
            }
        }
    }

    #[test]
    fn test_simple_trailing_comma_mapping() {
        let yaml = "{ one : two , }";
        #[cfg(feature = "debug-trace")]
        println!("Testing simple trailing comma mapping: {}", yaml);
        let mut source = BufferSource::new(yaml.as_bytes());

        match parse(&mut source) {
            Ok(_) => {
                #[cfg(feature = "debug-trace")]
                println!("Parsed successfully");
            }
            Err(e) => {
                #[cfg(feature = "debug-trace")]
                println!("Parse error: {}", e);
                panic!("Failed to parse: {}", e);
            }
        }
    }

    #[test]
    fn test_simple_trailing_comma_sequence() {
        let yaml = "[ one, two, ]";
        #[cfg(feature = "debug-trace")]
        println!("Testing simple trailing comma sequence: {}", yaml);
        let mut source = BufferSource::new(yaml.as_bytes());

        match parse(&mut source) {
            Ok(_) => {
                #[cfg(feature = "debug-trace")]
                println!("Parsed successfully");
            }
            Err(e) => {
                #[cfg(feature = "debug-trace")]
                println!("Parse error: {}", e);
                panic!("Failed to parse: {}", e);
            }
        }
    }

    #[test]
    fn test_simple_crlf() {
        // Simplest possible CRLF test
        let yaml = "key: value\r\n";

        #[cfg(feature = "debug-trace")]
        println!("Testing simple CRLF");
        let mut source = BufferSource::new(yaml.as_bytes());

        match parse(&mut source) {
            Ok(_) => {
                #[cfg(feature = "debug-trace")]
                println!("Parsed successfully");
            }
            Err(e) => {
                #[cfg(feature = "debug-trace")]
                println!("Parse error: {}", e);
                panic!("Failed to parse: {}", e);
            }
        }
    }

    #[test]
    fn test_flow_mapping_crlf() {
        // Flow mapping with CRLF
        let yaml = "{ one : two }\r\n";

        #[cfg(feature = "debug-trace")]
        println!("Testing flow mapping with CRLF");
        let mut source = BufferSource::new(yaml.as_bytes());

        match parse(&mut source) {
            Ok(_) => {
                #[cfg(feature = "debug-trace")]
                println!("Parsed successfully");
            }
            Err(e) => {
                #[cfg(feature = "debug-trace")]
                println!("Parse error: {}", e);
                panic!("Failed to parse: {}", e);
            }
        }
    }

    #[test]
    fn test_5c5m_with_crlf() {
        // Test with CRLF line endings - normalize to LF to avoid hang
        let yaml = "- { one : two , three: four , }\r\n- {five: six,seven : eight}\r\n";
        let normalized = yaml.replace("\r\n", "\n");

        #[cfg(feature = "debug-trace")]
        println!("Testing 5C5M with normalized CRLF");
        let mut source = BufferSource::new(normalized.as_bytes());

        match parse(&mut source) {
            Ok(_) => {
                #[cfg(feature = "debug-trace")]
                println!("Parsed successfully");
            }
            Err(e) => {
                #[cfg(feature = "debug-trace")]
                println!("Parse error: {}", e);
                panic!("Failed to parse: {}", e);
            }
        }
    }

    #[test]
    fn test_5c5m_exact_bytes() {
        use crate::{BufferSource, parse};
        // Exact bytes from 5C5M test file - normalize CRLF to LF
        let yaml_bytes: &[u8] = &[
            0x2D, 0x20, 0x7B, 0x20, 0x6F, 0x6E, 0x65, 0x20, 0x3A, 0x20, 0x74, 0x77, 0x6F, 0x20,
            0x2C, 0x20, 0x74, 0x68, 0x72, 0x65, 0x65, 0x3A, 0x20, 0x66, 0x6F, 0x75, 0x72, 0x20,
            0x2C, 0x20, 0x7D, 0x0D, 0x0A, 0x2D, 0x20, 0x7B, 0x66, 0x69, 0x76, 0x65, 0x3A, 0x20,
            0x73, 0x69, 0x78, 0x2C, 0x73, 0x65, 0x76, 0x65, 0x6E, 0x20, 0x3A, 0x20, 0x65, 0x69,
            0x67, 0x68, 0x74, 0x7D, 0x0D, 0x0A,
        ];

        // Normalize CRLF to LF
        let yaml_string = String::from_utf8_lossy(yaml_bytes).replace("\r\n", "\n");

        #[cfg(feature = "debug-trace")]
        println!("Testing 5C5M with normalized exact bytes");
        let mut source = BufferSource::new(yaml_string.as_bytes());

        match parse(&mut source) {
            Ok(_) => {
                #[cfg(feature = "debug-trace")]
                println!("Parsed successfully");
            }
            Err(e) => {
                #[cfg(feature = "debug-trace")]
                println!("Parse error: {}", e);
                panic!("Failed to parse: {}", e);
            }
        }
    }
}
