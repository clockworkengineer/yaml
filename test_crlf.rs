use yaml_lib::{parse, BufferSource};
use std::fs;

fn main() {
    // Read with CRLF
    let content_crlf = fs::read_to_string("tests/yaml-test-suite/7BMT/in.yaml").unwrap();
    // Convert to LF
    let content_lf = content_crlf.replace("\r\n", "\n");
    
    println!("Testing with CRLF:");
    let mut source = BufferSource::new(content_crlf.as_bytes());
    match parse(&mut source) {
        Ok(_) => println!("✓ OK"),
        Err(e) => println!("✗ {}", e),
    }
    
    println!("\nTesting with LF:");
    let mut source = BufferSource::new(content_lf.as_bytes());
    match parse(&mut source) {
        Ok(_) => println!("✓ OK"),
        Err(e) => println!("✗ {}", e),
    }
}
