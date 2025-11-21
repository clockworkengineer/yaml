use yaml_lib::{parse, BufferSource};

fn main() {
    // Test 26DV with CRLF
    let yaml = "\"top1\" : \r\n  \"key1\" : &alias1 scalar1\r\n'top2' : \r\n  'key2' : &alias2 scalar2\r\ntop3: &node3\r\n  *alias1 : scalar3\r\ntop4:\r\n  *alias2 : scalar4\r\ntop5   :\r\n  scalar5\r\ntop6:\r\n  &anchor6 'key6' : scalar6";
    let mut source = BufferSource::new(yaml.as_bytes());
    match parse(&mut source) {
        Ok(_doc) => {
            println!("SUCCESS with CRLF!");
        },
        Err(e) => println!("Error: {}", e),
    }
}
