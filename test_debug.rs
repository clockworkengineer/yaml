use yaml_lib::{parse, BufferSource};

fn main() {
    let yaml = "&k1 key1: one";
    println!("Parsing: {:?}", yaml);
    println!("Bytes: {:?}", yaml.as_bytes());
    
    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(doc) => {
            println!("Result: {:#?}", doc);
        },
        Err(e) => println!("Error: {}", e),
    }
}
