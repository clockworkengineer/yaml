use yaml_lib::{parse, BufferSource};

#[test]
fn test_9mma_directive_without_document() {
    // Test: %YAML directive with no document content
    let yaml = b"%YAML 1.2\n";
    let mut source = BufferSource::new(yaml);
    let result = parse(&mut source);
    match result {
        Err(e) => {
            println!("Error (expected): {}", e);
        }
        Ok(_) => {
            panic!("Should reject directive without document");
        }
    }
}

#[test]
fn test_b63p_directive_with_only_end_marker() {
    // Test: Directive followed by ... with no document content
    let yaml = b"%YAML 1.2\n...\n";
    let mut source = BufferSource::new(yaml);
    let result = parse(&mut source);
    match result {
        Err(e) => {
            println!("Error (expected): {}", e);
            assert!(
                e.contains("Directive must be followed by a document"),
                "Error message should mention directive requirement"
            );
        }
        Ok(_) => {
            panic!("Should reject directive with only end marker");
        }
    }
}

#[test]
fn test_valid_directive_with_document() {
    // Test: Proper directive followed by document content
    let yaml = b"%YAML 1.2\n---\nkey: value\n";
    let mut source = BufferSource::new(yaml);
    let result = parse(&mut source);
    match result {
        Ok(_) => {
            // Success
        }
        Err(e) => {
            panic!("Should parse valid directive with document, got error: {}", e);
        }
    }
}