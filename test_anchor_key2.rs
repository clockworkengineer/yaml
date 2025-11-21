use yaml_lib::{parse, BufferSource};

fn main() {
    let yaml = "&k1 key1: one";
    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(doc) => {
            println!("SUCCESS!");
            println!("{:#?}", doc);
        },
        Err(e) => println!("Error: {}", e),
    }
}
