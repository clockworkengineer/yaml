use yaml_lib::{BufferSource, parse};

fn main() {
    let input = "%YAML 1.2\n--- text\n";
    println!("Testing input: {:?}", input);
    
    let mut source = BufferSource::new(input.as_bytes());
    match parse(&mut source) {
        Ok(doc) => {
            println!("SUCCESS! Parsed as: {:?}", doc);
        }
        Err(e) => {
            println!("ERROR: {}", e);
        }
    }
}
