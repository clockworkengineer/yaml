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

    // Test C2DT - empty mapping value (should pass now)
    test_case("C2DT", b"{\n\"adjacent\":value,\n\"readable\": value,\n\"empty\":\n}", false);

    // Test 58MP - colon as value (should pass)
    test_case("58MP", b"{x: :x}", false);

    // Test 4ZYM - plain scalar with continuation lines
    test_case("4ZYM", b"plain: text\n  lines\nquoted: \"text\n  \tlines\"\nblock: |\n  text\n   \tlines", false);

    // Test 4JVG - scalar with two anchors (should fail)
    test_case("4JVG", b"top1: &node1\n  &k1 key1: val1\ntop2: &node2\n  &v2 val2", true);

    // Test 62EZ - invalid content after flow mapping (should fail, now fixed)
    test_case("62EZ", b"---\nx: { y: z }in: valid", true);

    // Test 4H7K - extra closing bracket (should fail)
    test_case("4H7K", b"---\n[ a, b, c ] ]", true);
}
