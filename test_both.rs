use yaml_lib::{parse, BufferSource};

fn main() {
    // Simpler test
    let yaml = "&k1 key1: one";
    
    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(doc) => {
            println!("SUCCESS for simple case!");
            println!("{:#?}", doc);
        },
        Err(e) => println!("Error: {}", e),
    }
    
    // Full test
    let yaml2 = r#"---
top1: &node1
  &k1 key1: one"#;
    
    let mut source2 = BufferSource::new(yaml2.as_bytes());
    match parse(&mut source2) {
        Ok(doc) => {
            println!("\nSUCCESS for full case!");
            println!("{:#?}", doc);
        },
        Err(e) => println!("\nError: {}", e),
    }
}
