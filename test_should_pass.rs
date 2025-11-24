use yaml_lib::{BufferSource, parse};

fn test_case(id: &str, yaml: &[u8]) {
    println!("\n=== {} ===", id);
    println!("Input: {}", String::from_utf8_lossy(yaml));
    
    let mut source = BufferSource::new(yaml);
    match parse(&mut source) {
        Ok(docs) => {
            println!("✓ SUCCESS");
        }
        Err(e) => {
            println!("✗ ERROR: {}", e);
        }
    }
}

fn main() {
    // FH7J: Tags on Empty Scalars
    test_case("FH7J", b"- !!str\n-\n  !!null : a\n  b: !!str\n- !!str : !!null\n");
    
    // PW8X: Anchors on Empty Scalars
    test_case("PW8X", b"- &a\n- a\n-\n  &a : a\n  b: &b\n-\n  &c : &a\n-\n  ? &d\n-\n  ? &e\n  : &a\n");
    
    // Y2GN: Anchor with colon in the middle
    test_case("Y2GN", b"---\nkey: &an:chor value\n");
    
    // 26DV: Whitespace around colon in mappings
    test_case("26DV", b"\"top1\" : \n  \"key1\" : &alias1 scalar1\n");
}
