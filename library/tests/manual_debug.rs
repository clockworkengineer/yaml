use std::fs;
use std::path::Path;
use yaml_lib::{parse, BufferSource};

#[test]
fn manual_debug_5tym() {
    let p = Path::new("c:/Projects/yaml/yaml-test-suite/5TYM/in.yaml");
    let s = fs::read_to_string(p).expect("read 5TYM");
    let mut src = BufferSource::new(s.as_bytes());
    match parse(&mut src) {
        Ok(node) => {
            println!("Parsed OK: {:?}", node);
        }
        Err(e) => {
            println!("5TYM parse error: {}", e);
        }
    }
}
