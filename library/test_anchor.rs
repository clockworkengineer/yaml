use yaml_lib::{parse, BufferSource};

fn main() {
    // Anchor on mapping key
    let yaml = "&outer\n&key [a]: value";
    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(doc) => {
            println!("SUCCESS!");
            println!("{:#?}", doc);
        },
        Err(e) => println!("Error: {}", e),
    }
}
