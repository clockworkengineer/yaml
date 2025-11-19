use std::fs;

fn main() {
    // Test empty flow collections
    let yaml_content = fs::read("c:/Projects/yaml/tests/yaml-test-suite/7ZZ5/in.yaml").unwrap();
    
    println!("Testing 7ZZ5 - Empty flow collections");
    println!("Input: {:?}", String::from_utf8_lossy(&yaml_content));
    
    // Use the library path
    let lib_path = "c:/Projects/yaml/library";
    std::env::set_current_dir(lib_path).unwrap();
    
    // Parse using the library
    use yaml_lib::io::sources::buffer::BufferSource;
    use yaml_lib::parser::parse;
    
    let mut source = BufferSource::new(&yaml_content);
    match parse(&mut source) {
        Ok(result) => {
            println!("SUCCESS: Parsed as {:?}", result);
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
}
