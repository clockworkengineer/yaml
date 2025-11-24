use yaml_lib::{BufferSource, parse};

fn test_tab(name: &str, yaml: &[u8], should_fail: bool) {
    let mut source = BufferSource::new(yaml);
    match parse(&mut source) {
        Ok(_) => {
            if should_fail {
                println!("❌ {}: PARSED (should fail)", name);
            } else {
                println!("✓ {}: PARSED (correct)", name);
            }
        }
        Err(_) => {
            if should_fail {
                println!("✓ {}: FAILED (correct)", name);
            } else {
                println!("❌ {}: FAILED (should parse)", name);
            }
        }
    }
}

fn main() {
    println!("Testing tab validation:\n");
    
    // 000: Tab in block scalar content (should fail)
    test_tab("Y79Y/000", b"foo: |\r\n\t\r\nbar: 1\r\n", true);
    
    // 001: Space+tab in block scalar (should pass)
    test_tab("Y79Y/001", b"foo: |\r\n \t\r\nbar: 1\r\n", false);
    
    // 002: Tab in flow sequence (should pass)
    test_tab("Y79Y/002", b"- [\r\n\t\r\n foo\r\n ]\r\n", false);
    
    // 003: Tab before flow content (should fail)
    test_tab("Y79Y/003", b"- [\r\n\tfoo,\r\n foo\r\n ]\r\n", true);
    
    // 004: Tab after sequence indicator (should fail)
    test_tab("Y79Y/004", b"-\t-\r\n", true);
    
    // 005: Space+tab after sequence indicator (should fail)
    test_tab("Y79Y/005", b"- \t-\r\n", true);
    
    // 006: Tab after mapping key indicator (should fail)
    test_tab("Y79Y/006", b"?\t-\r\n", true);
    
    // 007: Tab after mapping value indicator (should fail)
    test_tab("Y79Y/007", b"? -\r\n:\t-\r\n", true);
    
    // 008: Tab after explicit key indicator (should fail)
    test_tab("Y79Y/008", b"?\tkey:\r\n", true);
    
    // 009: Tab after explicit value indicator (should fail)
    test_tab("Y79Y/009", b"? key:\r\n:\tkey:\r\n", true);
    
    // 010: Tab after dash+content (should pass)
    test_tab("Y79Y/010", b"-\t-1\r\n", false);
}
