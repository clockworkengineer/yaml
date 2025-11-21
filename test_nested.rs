use yaml_lib::{parse, BufferSource};

fn test(name: &str, yaml: &str) {
    println!("\n=== {} ===", name);
    println!("Input: {:?}", yaml);
    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(doc) => println!("✓ Success: {:#?}", doc),
        Err(e) => println!("✗ Error: {}", e),
    }
}

fn main() {
    // These should work
    test("simple anchor", "&a value");
    test("anchor on value", "key: &a value");
    test("anchor on key (root)", "&k key: value");
    
    // This is the problem case
    test("nested anchors", "top: &node\n  &k key: value");
}
