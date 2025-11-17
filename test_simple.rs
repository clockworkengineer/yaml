use yaml_lib::io::sources::buffer::Buffer;
use yaml_lib::parser::document::parse;

fn main() {
    let yaml = b"%YAML 1.2\n--- text\n";
    let mut source = Buffer::new(yaml);
    match parse(&mut source) {
        Ok(doc) => println!("OK: {:?}", doc),
        Err(e) => println!("ERROR: {}", e),
    }
}
