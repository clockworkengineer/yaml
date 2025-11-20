use yaml_lib::{parse, BufferSource};

fn main() {
    // Anchor inside inline sequence
    let yaml = "&key [ &item a, b, c ]";
    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(doc) => {
            println!("SUCCESS!");
            println!("{:#?}", doc);
        },
        Err(e) => println!("Error: {}", e),
    }
}
