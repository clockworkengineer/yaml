use yaml_lib::{BufferSource, parse};
use std::fs;
use std::time::{Duration, Instant};

fn test_with_timeout(id: &str, timeout_secs: u64) -> Result<bool, String> {
    let path = format!("tests/yaml-test-suite/{}/in.yaml", id);
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path, e))?;
    
    let start = Instant::now();
    let mut source = BufferSource::new(content.as_bytes());
    let result = parse(&mut source);
    let elapsed = start.elapsed();
    
    if elapsed.as_secs() > timeout_secs {
        return Err(format!("TIMEOUT after {:?}", elapsed));
    }
    
    Ok(result.is_ok())
}

fn main() {
    let test_ids = vec!["FH7J", "PW8X", "EB22", "EHF6"];
    
    for id in test_ids {
        print!("{}: ", id);
        match test_with_timeout(id, 5) {
            Ok(true) => println!("✓ PASS"),
            Ok(false) => println!("✗ FAIL"),
            Err(e) => println!("⏱ {}", e),
        }
    }
}
