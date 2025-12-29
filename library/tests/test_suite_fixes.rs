//! Quick test for specific failing test cases
use yaml_lib::{BufferSource, parse};

fn test_case(id: &str, yaml: &[u8], should_error: bool) {
    let mut source = BufferSource::new(yaml);
    let result = parse(&mut source);
    let passed = if should_error {
        result.is_err()
    } else {
        result.is_ok()
    };

    println!("{}: {} (expected: {}, got: {})",
        id,
        if passed { "PASS" } else { "FAIL" },
        if should_error { "error" } else { "success" },
        if result.is_ok() { "success" } else { "error" }
    );

    if !passed {
        if let Err(e) = result {
            println!("  Error: {}", e);
        }
    }
}

#[test]
fn test_specific_failures() {
    // Test 4H7K - extra closing bracket
    test_case("4H7K", b"---\n[ a, b, c ] ]\n", true);

    // Test 236B - invalid value after mapping
    test_case("236B", b"foo:\n  bar\ninvalid\n", true);

    // Test 55WF - invalid escape (should now fail correctly)
    test_case("55WF", b"---\n\"\\.\"", true);
}
