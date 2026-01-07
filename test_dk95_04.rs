use yaml_lib::{parse, BufferSource};

fn main() {
    let yaml = "foo: 1\n\t\nbar: 2\n";
    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(doc) => {
            println!("Success: {:?}", doc);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
