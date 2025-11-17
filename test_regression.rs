use yaml_lib::{BufferSource, parse};

fn test_case(name: &str, input: &str) {
    println!("\nTesting {}: {:?}", name, input);
    let mut source = BufferSource::new(input.as_bytes());
    match parse(&mut source) {
        Ok(doc) => println!("  ✓ SUCCESS"),
        Err(e) => println!("  ✗ ERROR: {}", e),
    }
}

fn main() {
    // Test cases that were regressing
    test_case("27NA (directive)", "%YAML 1.2\n--- text\n");
    test_case("simple scalar", "--- hello\n");
    test_case("mapping with colon", "--- key: value\n");
    
    // Test cases that should error (our validation tests)
    test_case("7MNF (missing colon)", "top1:\n  key1: val1\ntop2\n");
}
