use yaml_lib::{BufferSource, parse};

fn test_case(name: &str, yaml: &[u8], should_fail: bool) {
    print!("{}: ", name);
    let mut source = BufferSource::new(yaml);
    match parse(&mut source) {
        Ok(_) => {
            if should_fail {
                println!("❌ PARSED (should fail)");
            } else {
                println!("✓ PARSED (correct)");
            }
        }
        Err(e) => {
            if should_fail {
                println!("✓ FAILED (correct): {}", e);
            } else {
                println!("❌ FAILED (should parse): {}", e);
            }
        }
    }
}

fn main() {
    println!("Testing directive validation:\n");
    
    test_case("EB22 (missing ... before directive)", 
        b"---\nscalar1 # comment\n%YAML 1.2\n---\nscalar2\n", true);
    
    test_case("H7TQ (extra words after version)", 
        b"%YAML 1.2 foo\n---\n", true);
    
    test_case("SF5V (duplicate YAML directive)", 
        b"%YAML 1.2\n%YAML 1.2\n---\n", true);
    
    test_case("MUS6/00 (no space before comment)", 
        b"%YAML 1.1#...\n---\n", true);
}
