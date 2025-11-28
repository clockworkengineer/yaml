#[cfg(test)]
mod debug_flow_hang {
    use crate::{BufferSource, parse};

    #[test]
    #[ignore] // Run manually with: cargo test debug_simple_case -- --ignored --nocapture
    fn debug_simple_case() {
        eprintln!("\n=== Testing: - {{ a: 1, }} ===");
        let yaml = b"- { a: 1, }";
        let mut source = BufferSource::new(yaml);

        eprintln!("Starting parse...");
        match parse(&mut source) {
            Ok(node) => {
                eprintln!("✓ SUCCESS: Parsed without hanging!");
                eprintln!("Result: {:?}", node);
            }
            Err(e) => {
                eprintln!("✗ ERROR: {}", e);
            }
        }
    }

    #[test]
    #[ignore]
    fn debug_with_newline() {
        eprintln!("\n=== Testing: - {{ a: 1, }}\\n ===");
        let yaml = b"- { a: 1, }\n";
        let mut source = BufferSource::new(yaml);

        eprintln!("Starting parse...");
        match parse(&mut source) {
            Ok(node) => {
                eprintln!("✓ SUCCESS: Parsed without hanging!");
                eprintln!("Result: {:?}", node);
            }
            Err(e) => {
                eprintln!("✗ ERROR: {}", e);
            }
        }
    }

    #[test]
    #[ignore]
    fn debug_two_items() {
        eprintln!("\n=== Testing: two items ===");
        let yaml = b"- { a: 1, }\n- { b: 2 }";
        let mut source = BufferSource::new(yaml);

        eprintln!("Starting parse...");
        match parse(&mut source) {
            Ok(node) => {
                eprintln!("✓ SUCCESS: Parsed without hanging!");
                eprintln!("Result: {:?}", node);
            }
            Err(e) => {
                eprintln!("✗ ERROR: {}", e);
            }
        }
    }
}
