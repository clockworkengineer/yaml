use yaml_lib::{BufferSource, parse};

fn main() {
    let input = b"key1: &alias value1\n&b *alias : value2";
    let mut source = BufferSource::new(input);
    
    match parse(&mut source) {
        Ok(docs) => {
            println!("Parsed successfully (but should ERROR!):");
            println!("{:#?}", docs);
        }
        Err(e) => {
            println!("Error (expected): {}", e);
        }
    }
}
