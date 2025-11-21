use yaml_lib::{parse, BufferSource};

fn main() {
    // Test tab in double quoted string
    let yaml = r#""2 leading
    \	tab""#;
    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(doc) => {
            println!("SUCCESS!");
            println!("{:#?}", doc);
        },
        Err(e) => println!("Error: {}", e),
    }
}
