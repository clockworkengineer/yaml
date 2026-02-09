use yaml_lib::parse;
use yaml_lib::BufferSource;

#[test]
fn debug_ugm3_should_succeed() {
    let input = b"--- !<tag:clarkevans.com,2002:invoice>\ninvoice: 34843\n";
    let mut source = BufferSource::new(input);
    match parse(&mut source) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("UGM3 parse error: {}", e);
            panic!("UGM3 should succeed");
        }
    }
}

#[test]
fn debug_7fwl_should_succeed() {
    let input = b"!<tag:yaml.org,2002:str> foo :\n  !<!bar> baz\n";
    let mut source = BufferSource::new(input);
    match parse(&mut source) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("7FWL parse error: {}", e);
            panic!("7FWL should succeed");
        }
    }
}
