use yaml_lib::{BufferSource, parse};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <test_name>", args[0]);
        std::process::exit(1);
    }
    
    let test_name = &args[1];
    let test_dir = format!("tests/yaml-test-suite/{}", test_name);
    
    // Check if test directory exists
    if !Path::new(&test_dir).exists() {
        eprintln!("Test directory not found: {}", test_dir);
        std::process::exit(1);
    }
    
    // Read test name from === file
    let name_file = format!("{}/===", test_dir);
    let test_description = fs::read_to_string(&name_file)
        .unwrap_or_else(|_| String::from("Unknown test"));
    
    println!("=== Test Case: {} ===", test_name);
    println!("Name: {}", test_description.trim());
    
    // Determine if test should pass or fail
    let should_fail = Path::new(&format!("{}/error", test_dir)).exists();
    let has_in_yaml = Path::new(&format!("{}/in.yaml", test_dir)).exists();
    
    println!("Should fail: {}", should_fail);
    println!();
    
    // Read and parse YAML
    let in_yaml = format!("{}/in.yaml", test_dir);
    if !has_in_yaml {
        println!("No in.yaml file - skipping");
        println!("\n✓ Status: PASS (correctly skipped)");
        return;
    }
    
    let yaml_content = fs::read_to_string(&in_yaml)
        .expect("Failed to read in.yaml");
    
    println!("--- Input YAML ---");
    println!("{}", yaml_content);
    println!("\n--- End Input ---");
    println!();
    
    let mut source = BufferSource::new(yaml_content.as_bytes());
    let result = parse(&mut source);
    
    match result {
        Ok(node) => {
            println!("✓ Parsing SUCCEEDED");
            println!("\n--- Parsed Structure ---");
            println!("{:#?}", node);
            
            if should_fail {
                println!("\n⚠️  WARNING: This test should have FAILED but succeeded");
                println!("Status: FAIL");
            } else {
                println!("\nStatus: PASS");
            }
        }
        Err(e) => {
            println!("✗ Parsing FAILED");
            println!("\n--- Error ---");
            println!("{}", e);
            
            if !should_fail {
                println!("\n⚠️  WARNING: This test should have SUCCEEDED but failed");
                println!("Status: FAIL");
            } else {
                println!("\nStatus: PASS (correctly failed)");
            }
        }
    }
}
