use yaml_lib::{parse, BufferSource};

fn main() {
    let yaml = r#"- !!str
-
  !!null : a
  b: !!str
- !!str : !!null
"#;
    
    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(node) => {
            println!("SUCCESS: Parsed FH7J pattern!");
            println!("Result: {:?}", node);
        }
        Err(e) => {
            println!("ERROR: Failed to parse FH7J");
            println!("Error: {}", e);
        }
    }
}
