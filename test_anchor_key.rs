use yaml_lib::{parse, BufferSource};

fn main() {
    // Simple: anchor on a mapping key
    let yaml = "top: &k1 key1: value";
    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(doc) => {
            println!("SUCCESS!");
            println!("{:#?}", doc);
        },
        Err(e) => println!("Error: {}", e),
    }
}
