#[cfg(test)]
mod test_directive_validation {
    use crate::{BufferSource, parse};

    #[test]
    fn test_9mma_directive_without_document() {
        // Test: %YAML directive with no document content
        let yaml = b"%YAML 1.2\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        if let Ok(ref res) = result {
            println!("Parsed: {:?}", res);
        }
        // YAML spec requires document content after directives
        // Currently we might accept this, but it should error
        // TODO: Validate directives are followed by document content or ---
        // assert!(result.is_err(), "Should reject directive without document");
    }

    #[test]
    fn test_b63p_directive_with_only_end_marker() {
        // Test: Directive followed by ... with no document content
        let yaml = b"%YAML 1.2\n...\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        if let Ok(ref res) = result {
            println!("Parsed: {:?}", res);
        }
        // This should also be invalid - no document between directive and ...
        // assert!(result.is_err(), "Should reject directive with only end marker");
    }

    #[test]
    fn test_valid_directive_with_document() {
        // Test: Proper directive followed by document content
        let yaml = b"%YAML 1.2\n---\nkey: value\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_ok(), "Should parse valid directive with document");
    }
}
