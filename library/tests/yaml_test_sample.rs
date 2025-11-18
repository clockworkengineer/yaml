//! Sample test to verify YAML test suite loading works

use std::fs;
use std::path::Path;
use std::panic;
use std::time::{Duration, Instant};
use yaml_lib::{BufferSource, parse};

#[test]
fn test_sample_yaml_cases() {
    let suite_dir = Path::new("../tests/yaml-test-suite");

    if !suite_dir.exists() {
        println!("YAML test suite not found");
        return;
    }

    // Test just a few specific cases
    let test_ids = vec!["229Q", "236B", "26DV", "27NA", "2JQS"];
    
    let mut passed = 0;
    let mut failed = 0;
    let mut timeouts = 0;

    for test_id in test_ids {
        let test_dir = suite_dir.join(test_id);
        
        if !test_dir.exists() {
            println!("Test {} not found", test_id);
            continue;
        }

        // Read test files
        let name = fs::read_to_string(test_dir.join("==="))
            .unwrap_or_default()
            .trim()
            .to_string();
        let yaml = match fs::read_to_string(test_dir.join("in.yaml")) {
            Ok(y) => y,
            Err(_) => {
                println!("Could not read {}", test_id);
                continue;
            }
        };
        let has_error = test_dir.join("error").exists();

        println!("\nTesting {}: {}", test_id, name);
        println!("  Expected: {}", if has_error { "error" } else { "success" });

        // Run with timeout
        let start = Instant::now();
        let result = panic::catch_unwind(|| {
            let mut source = BufferSource::new(yaml.as_bytes());
            parse(&mut source)
        });
        let elapsed = start.elapsed();

        println!("  Elapsed: {:?}", elapsed);

        if elapsed > Duration::from_millis(100) {
            println!("  TIMEOUT!");
            timeouts += 1;
            continue;
        }

        let parse_result = match result {
            Ok(r) => r,
            Err(_) => {
                println!("  PANIC!");
                failed += 1;
                continue;
            }
        };

        let test_passed = match (parse_result.is_ok(), has_error) {
            (true, false) => true,
            (false, true) => true,
            _ => false,
        };

        println!("  Got: {}", if parse_result.is_ok() { "success" } else { "error" });
        
        if test_passed {
            println!("  ✓ PASS");
            passed += 1;
        } else {
            println!("  ✗ FAIL");
            failed += 1;
        }
    }

    println!("\n=== Summary ===");
    println!("Passed:   {}", passed);
    println!("Failed:   {}", failed);
    println!("Timeouts: {}", timeouts);
}
