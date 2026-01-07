use yaml_lib::{parse, BufferSource};

fn main() {
    // DK95/04 test case: tab-only line in block mapping
    let yaml = "foo: 1\n\t\nbar: 2\n";
    println!("Testing: {:?}", yaml);
    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(doc) => {
            println!("✓ Success! Parsed correctly.");
            println!("Document: {:?}", doc);
        }
        Err(e) => {
            println!("✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}
