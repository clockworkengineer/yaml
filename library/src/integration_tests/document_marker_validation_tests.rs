#[cfg(test)]
mod test_document_marker_validation {
    use crate::{BufferSource, parse};

    #[test]
    fn test_9kbc_mapping_on_document_start_line() {
        // Test: Content on same line as --- is invalid
        // YAML spec requires --- to be on its own line
        let yaml = b"--- key1: value1\n    key2: value2\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        if let Err(ref _e) = result {
            #[cfg(feature = "debug-trace")]
            println!("Error (correct): {}", _e);
        }
        assert!(result.is_err(), "Should reject content on --- line");
    }

    #[test]
    fn test_valid_document_marker() {
        // Test: Correct usage with --- on separate line
        let yaml = b"---\nkey1: value1\nkey2: value2\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);
        assert!(result.is_ok(), "Should parse valid document marker");
    }
}
