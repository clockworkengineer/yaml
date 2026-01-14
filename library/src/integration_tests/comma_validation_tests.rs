#[cfg(test)]
mod test_comma_validation {
    use crate::{BufferSource, parse};

    #[test]
    fn test_9mag_leading_comma_in_sequence() {
        // Test: Leading comma in flow sequence (invalid)
        let yaml = b"---\n[ , a, b, c ]\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "Should reject leading comma in flow sequence"
        );
        if let Err(e) = result {
            assert!(e.to_string().contains("comma"), "Error should mention comma: {}", e);
        }
    }

    #[test]
    fn test_ctn5_double_comma_in_sequence() {
        // Test: Double comma in flow sequence (invalid)
        let yaml = b"---\n[ a, b, c, , ]\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "Should reject double comma in flow sequence"
        );
        if let Err(e) = result {
            let e_str = e.to_string();
            assert!(
                e_str.contains("comma") || e_str.contains("consecutive"),
                "Error should mention comma: {}",
                e_str
            );
        }
    }

    #[test]
    fn test_cvw2_comment_without_space_after_comma() {
        // Test: Comment directly after comma with no space (invalid)
        // YAML spec requires whitespace before #
        let yaml = b"---\n[ a, b, c,#invalid\n]\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        if let Err(ref _e) = result {
            #[cfg(feature = "debug-trace")]
            println!("Error (correct): {}", _e);
        }
        assert!(
            result.is_err(),
            "Should reject comment without whitespace after comma"
        );
    }

    #[test]
    fn test_9jba_comment_without_space_after_bracket() {
        // Test: Comment directly after ] with no space (invalid)
        let yaml = b"---\n[ a, b, c, ]#invalid\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        if let Err(ref _e) = result {
            #[cfg(feature = "debug-trace")]
            println!("Error (correct): {}", _e);
        } else {
            #[cfg(feature = "debug-trace")]
            println!("Parsed (should be error): {:?}", result);
        }
        assert!(
            result.is_err(),
            "Should reject comment without whitespace after ]"
        );
    }

    #[test]
    #[ignore] // TODO: This requires detecting ": " pattern in plain scalars in flow context
    fn test_t833_missing_comma_in_mapping() {
        // Test: Flow mapping missing separating comma (invalid)
        // Currently fails because the value parser collects "1 bar: 2" as a single value
        // Proper fix requires stopping plain scalar collection at ": " pattern in flow context
        let yaml = b"---\n{\n foo: 1\n bar: 2 }\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        if result.is_ok() {
            #[cfg(feature = "debug-trace")]
            println!("Result: {:?}", result);
        }
        assert!(
            result.is_err(),
            "Should reject flow mapping missing comma between pairs"
        );
    }

    #[test]
    #[ignore] // TODO: Same issue as t833 - requires flow context plain scalar restrictions
    fn test_t833_simplified() {
        // Simplified version without line breaks
        // Parser treats "1 bar: 2" as a single plain scalar value
        let yaml = b"{foo: 1 bar: 2}";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        if result.is_ok() {
            #[cfg(feature = "debug-trace")]
            println!("Simplified result: {:?}", result);
        }
        assert!(result.is_err(), "Should reject flow mapping missing comma");
    }

    #[test]
    fn test_5c5m_valid_trailing_comma() {
        // Test: Trailing comma (valid - should pass)
        let yaml = b"- { one : two , three: four , }\n- {five: six,seven : eight}";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_ok(),
            "Should accept trailing comma in flow mapping"
        );
    }

    #[test]
    fn test_valid_sequence_trailing_comma() {
        // Test: Trailing comma in sequence (valid - should pass)
        let yaml = b"[1, 2, 3,]";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_ok(),
            "Should accept trailing comma in flow sequence"
        );
    }
}
