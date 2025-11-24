use yaml_lib::{BufferSource, parse};

fn main() {
    println!("=== EB22 ===");
    let yaml = b"---\nscalar1 # comment\n%YAML 1.2\n---\nscalar2\n";
    let mut source = BufferSource::new(yaml);
    match parse(&mut source) {
        Ok(docs) => println!("Parsed: {:#?}", docs),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\n=== H7TQ ===");
    let yaml2 = b"%YAML 1.2 foo\n---\n";
    let mut source2 = BufferSource::new(yaml2);
    match parse(&mut source2) {
        Ok(docs) => println!("Parsed: {:#?}", docs),
        Err(e) => println!("Error: {}", e),
    }
}
