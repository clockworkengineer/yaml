use crate::io::sources::buffer::Buffer;
use crate::parser::document::parse;

#[test]
fn test_fh7j_pattern() {
    let yaml = b"- !!str\n";
    let mut source = Buffer::new(yaml);
    
    match parse(&mut source) {
        Ok(node) => {
            println!("Parsed successfully: {:?}", node);
        }
        Err(e) => {
            println!("Parse error: {}", e);
            panic!("Failed to parse FH7J pattern");
        }
    }
}

#[test]
fn test_pw8x_pattern() {
    let yaml = b"- &a\n";
    let mut source = Buffer::new(yaml);
    
    match parse(&mut source) {
        Ok(node) => {
            println!("Parsed successfully: {:?}", node);
        }
        Err(e) => {
            println!("Parse error: {}", e);
            panic!("Failed to parse PW8X pattern");
        }
    }
}
