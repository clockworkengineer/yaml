use yaml_lib::*;

fn main() {
    let yaml = "&a: test";
    let mut source = io::sources::buffer::Buffer::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(doc) => println!("Success: {:?}", doc),
        Err(e) => println!("Error: {}", e),
    }
}
