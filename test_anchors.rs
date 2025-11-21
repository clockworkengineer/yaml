use yaml_lib::{parse, BufferSource};
use std::fs;

fn test_file(name: &str) {
    let path = format!("tests/yaml-test-suite/{}/in.yaml", name);
    let content = fs::read_to_string(&path).unwrap();
    
    println!("\n=== Testing {} ===", name);
    println!("Input:\n{}", &content[..content.len().min(200)]);
    
    let mut source = BufferSource::new(content.as_bytes());
    match parse(&mut source) {
        Ok(doc) => {
            println!("✓ PARSED successfully");
            println!("Structure: {:#?}", &format!("{:#?}", doc)[..format!("{:#?}", doc).len().min(300)]);
        },
        Err(e) => println!("✗ Parse error: {}", e),
    }
}

fn main() {
    test_file("6BFJ");
    test_file("7BMT");
    test_file("U3XV");
    test_file("ZWK4");
    test_file("PW8X");
}
