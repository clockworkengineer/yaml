#[cfg(test)]
mod debug_flow_hang {
    use crate::test_helpers::parse_yaml;

    #[test]
    #[ignore] // Run manually with: cargo test debug_simple_case -- --ignored --nocapture
    fn debug_simple_case() {
        #[cfg(feature = "debug-trace")]
        eprintln!("\n=== Testing: - {{ a: 1, }} ===");
        let yaml = b"- { a: 1, }";
        let _node = parse_yaml(yaml);

        #[cfg(feature = "debug-trace")]
        eprintln!("Starting parse...");
        #[cfg(feature = "debug-trace")]
        eprintln!("✓ SUCCESS: Parsed without hanging!");
        #[cfg(feature = "debug-trace")]
        eprintln!("Result: {:?}", _node);
        // Handle errors if necessary
    }

    #[test]
    #[ignore]
    fn debug_with_newline() {
        #[cfg(feature = "debug-trace")]
        eprintln!("\n=== Testing: - {{ a: 1, }}\\n ===");
        let yaml = b"- { a: 1, }\n";
        let _node = parse_yaml(yaml);

        #[cfg(feature = "debug-trace")]
        eprintln!("Starting parse...");
        #[cfg(feature = "debug-trace")]
        eprintln!("✓ SUCCESS: Parsed without hanging!");
        #[cfg(feature = "debug-trace")]
        eprintln!("Result: {:?}", _node);
        // Handle errors if necessary
    }

    #[test]
    #[ignore]
    fn debug_two_items() {
        #[cfg(feature = "debug-trace")]
        eprintln!("\n=== Testing: two items ===");
        let yaml = b"- { a: 1, }\n- { b: 2 }";
        let _node = parse_yaml(yaml);

        #[cfg(feature = "debug-trace")]
        eprintln!("Starting parse...");
        #[cfg(feature = "debug-trace")]
        eprintln!("✓ SUCCESS: Parsed without hanging!");
        #[cfg(feature = "debug-trace")]
        eprintln!("Result: {:?}", _node);
        // Handle errors if necessary
    }
}
