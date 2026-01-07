#[cfg(test)]
mod debug_flow_hang {
    use crate::{BufferSource, parse};

    #[test]
    #[ignore] // Run manually with: cargo test debug_simple_case -- --ignored --nocapture
    fn debug_simple_case() {
        #[cfg(feature = "debug-trace")]
        eprintln!("\n=== Testing: - {{ a: 1, }} ===");
        let yaml = b"- { a: 1, }";
        let mut source = BufferSource::new(yaml);

        #[cfg(feature = "debug-trace")]
        eprintln!("Starting parse...");
        match parse(&mut source) {
            Ok(_node) => {
                #[cfg(feature = "debug-trace")]
                eprintln!("✓ SUCCESS: Parsed without hanging!");
                #[cfg(feature = "debug-trace")]
                eprintln!("Result: {:?}", _node);
            }
            Err(_e) => {
                #[cfg(feature = "debug-trace")]
                eprintln!("✗ ERROR: {}", _e);
            }
        }
    }

    #[test]
    #[ignore]
    fn debug_with_newline() {
        #[cfg(feature = "debug-trace")]
        eprintln!("\n=== Testing: - {{ a: 1, }}\\n ===");
        let yaml = b"- { a: 1, }\n";
        let mut source = BufferSource::new(yaml);

        #[cfg(feature = "debug-trace")]
        eprintln!("Starting parse...");
        match parse(&mut source) {
            Ok(_node) => {
                #[cfg(feature = "debug-trace")]
                eprintln!("✓ SUCCESS: Parsed without hanging!");
                #[cfg(feature = "debug-trace")]
                eprintln!("Result: {:?}", _node);
            }
            Err(_e) => {
                #[cfg(feature = "debug-trace")]
                eprintln!("✗ ERROR: {}", _e);
            }
        }
    }

    #[test]
    #[ignore]
    fn debug_two_items() {
        #[cfg(feature = "debug-trace")]
        eprintln!("\n=== Testing: two items ===");
        let yaml = b"- { a: 1, }\n- { b: 2 }";
        let mut source = BufferSource::new(yaml);

        #[cfg(feature = "debug-trace")]
        eprintln!("Starting parse...");
        match parse(&mut source) {
            Ok(_node) => {
                #[cfg(feature = "debug-trace")]
                eprintln!("✓ SUCCESS: Parsed without hanging!");
                #[cfg(feature = "debug-trace")]
                eprintln!("Result: {:?}", _node);
            }
            Err(_e) => {
                #[cfg(feature = "debug-trace")]
                eprintln!("✗ ERROR: {}", _e);
            }
        }
    }
}
