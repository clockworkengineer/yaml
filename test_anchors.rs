use yaml_lib::{BufferSource, parse};
use std::fs;

fn main() {
    let tests = vec!["G9HC", "GT5M", "H7J7", "SR86", "SU74", "SY6V"];
    
    for test_id in tests {
        let test_dir = format!("tests/yaml-test-suite/{}", test_id);
        let yaml = fs::read_to_string(format!("{}/in.yaml", test_dir)).unwrap();
        let name = fs::read_to_string(format!("{}/===", test_dir)).unwrap();
        
        println!("\n=== {} - {} ===", test_id, name.trim());
        println!("Input:\n{}", yaml);
        
        let mut source = BufferSource::new(yaml.as_bytes());
        match parse(&mut source) {
            Ok(doc) => {
                println!("✗ PARSED (but should ERROR)");
                println!("Result: {:?}", doc);
            }
            Err(e) => {
                println!("✓ CORRECTLY FAILED: {}", e);
            }
        }
    }
}
