use yaml_lib::{parse, BufferSource};
use std::fs;

fn main() {
    let content = fs::read_to_string("tests/yaml-test-suite/7BMT/in.yaml").unwrap();
    println!("Testing 7BMT:");
    println!("{}", content);
    println!("\n---\n");
    
    let mut source = BufferSource::new(content.as_bytes());
    match parse(&mut source) {
        Ok(doc) => {
            println!("✓ SUCCESS");
            println!("{:#?}", &format!("{:#?}", doc)[..500]);
        },
        Err(e) => println!("✗ Error: {}", e),
    }
}
