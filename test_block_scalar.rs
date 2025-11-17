use yaml_lib::{BufferSource, parse};

fn test_case(name: &str, input: &str, should_error: bool) {
    println!("\nTesting {}: {:?}", name, input);
    let mut source = BufferSource::new(input.as_bytes());
    match parse(&mut source) {
        Ok(_) => {
            if should_error {
                println!("  ✗ ERROR: Should have failed but passed");
            } else {
                println!("  ✓ SUCCESS");
            }
        }
        Err(e) => {
            if should_error {
                println!("  ✓ CORRECT ERROR: {}", e);
            } else {
                println!("  ✗ ERROR: Should have passed but failed: {}", e);
            }
        }
    }
}

fn main() {
    // Valid cases
    test_case("literal valid", "--- |\n  text\n", false);
    test_case("literal with indent", "--- |1\n  text\n", false);
    test_case("literal with chomp", "--- |-\n  text\n", false);
    test_case("literal with both", "--- |1-\n  text\n", false);
    test_case("folded valid", "--- >\n  text\n", false);
    
    // Invalid cases (should error)
    test_case("2G84/00 - indent 0", "--- |0\n", true);
    test_case("2G84/01 - indent 10", "--- |10\n", true);
    test_case("invalid modifier", "--- |x\n", true);
}
