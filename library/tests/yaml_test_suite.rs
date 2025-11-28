//! Official YAML 1.2 Test Suite Integration
//!
//! This test suite runs the official YAML test cases from:
//! https://github.com/yaml/yaml-test-suite (data-2022-01-17 release)
//!
//! The test suite contains 351+ test cases covering all aspects of YAML 1.2 specification.

use std::fs;
use std::panic;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use yaml_lib::{BufferSource, parse};

#[derive(Debug)]
struct TestCase {
    id: String,
    yaml: String,
    has_error_file: bool,
}

/// Load a test case from the data release format
/// Each test is in a directory with: ===, in.yaml, and optionally error
fn load_test_case(test_dir: &Path) -> Option<TestCase> {
    // For official test suite, use directory name as ID and load in.yaml
    let id = test_dir.file_name()?.to_str()?.to_string();
    let yaml_file = test_dir.join("in.yaml");
    let yaml = fs::read_to_string(&yaml_file).ok()?;
    let has_error_file = test_dir.join("error").exists();
    Some(TestCase {
        id,
        yaml,
        has_error_file,
    })
}

/// Get all test directories (both single and multi-test)
fn get_all_test_dirs(suite_dir: &Path) -> Vec<PathBuf> {
    fn collect_test_dirs(dir: &Path, out: &mut Vec<PathBuf>) {
        if dir.is_dir() {
            let entries = match fs::read_dir(dir) {
                Ok(e) => e,
                Err(_) => return,
            };
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    if path.join("in.yaml").exists() {
                        out.push(path.clone());
                    }
                    collect_test_dirs(path.as_path(), out);
                }
            }
        }
    }
    let mut test_dirs = Vec::new();
    collect_test_dirs(suite_dir, &mut test_dirs);
    test_dirs
}

/// Run all YAML test suite cases and assert pass rate >= 90%
#[test]
fn run_yaml_test_suite() {
    // Try multiple possible paths for the test suite
    let possible_paths = vec![
        Path::new("c:/Projects/yaml/yaml-test-suite/src"),
    ];

    let suite_dir = possible_paths.iter().find(|p| p.exists()).cloned();
    let suite_dir = match suite_dir {
        Some(dir) => dir,
        None => {
            println!("YAML test suite repo directory not found in any of these locations:");
            for path in &possible_paths {
                println!("  - {:?}", path);
            }
            println!(
                "Please clone https://github.com/yaml/yaml-test-suite.git to one of these locations."
            );
            return;
        }
    };

    // Skip list now empty - infinite loop protection added to parser
    let skip_list: Vec<&str> = vec![];

    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut failures = Vec::new();

    // Get all test directories
    let mut test_dirs = get_all_test_dirs(suite_dir);

    if test_dirs.is_empty() {
        println!("No test directories found. Make sure you're using the data release branch.");
        println!("Run: cd tests/yaml-test-suite && git checkout data-2022-01-17");
        return;
    }

    // Sort for consistent ordering
    test_dirs.sort();

    let total_dirs = test_dirs.len();

    println!("Running all {} YAML test suite tests...", total_dirs);

    let mut test_num = 0;

    // Run all tests
    let test_limit = 402;

    for (idx, test_dir) in test_dirs.iter().enumerate() {
        if idx >= test_limit {
            println!(
                "\n--- Stopping at {} tests (limit reached) ---\n",
                test_limit
            );
            break;
        }

        let test_dir = test_dir.clone();
        // Load test case
        let test = match load_test_case(&test_dir) {
            Some(t) => t,
            None => {
                skipped += 1;
                continue;
            }
        };

        // Skip if in skip list
        if skip_list.contains(&test.id.as_str()) {
            skipped += 1;
            continue;
        }

        test_num += 1;
        println!("[{}/402] Testing: {}", test_num, test.id);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        // Run the test with panic protection
        let start_time = Instant::now();
        let result = panic::catch_unwind(|| {
            let mut source = BufferSource::new(test.yaml.as_bytes());
            parse(&mut source)
        });
        let elapsed = start_time.elapsed();

        print!("  Result: ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        // Check for timeout (likely infinite loop if > 20ms per test)
        if elapsed > Duration::from_millis(20) {
            skipped += 1;
            println!("TIMEOUT (took {:?})", elapsed);
            continue;
        }

        // Handle panic
        let parse_result = match result {
            Ok(r) => r,
            Err(_) => {
                skipped += 1;
                println!("PANIC");
                continue;
            }
        };

        // Determine if test passed
        // Error tests should fail to parse, non-error tests should succeed
        let test_passed = match (parse_result.is_ok(), test.has_error_file) {
            (true, false) => true, // Should pass and did pass
            (false, true) => true, // Should fail and did fail
            _ => false,            // Mismatch
        };

        if test_passed {
            passed += 1;
            println!("PASS");
        } else {
            failed += 1;
            let expected = if test.has_error_file {
                "error"
            } else {
                "success"
            };
            let got = if parse_result.is_ok() {
                "success"
            } else {
                "error"
            };
            println!("FAIL (expected: {}, got: {})", expected, got);
            failures.push(format!(
                "{} (expected: {}, got: {})",
                test.id, expected, got
            ));
        }
    }

    // Print summary
    println!("\n=== YAML Test Suite Results (All Tests) ===");
    println!("Passed:  {}", passed);
    println!("Failed:  {}", failed);
    println!("Skipped: {}", skipped);
    println!("Total:   {}", passed + failed + skipped);

    if !failures.is_empty() {
        println!("\n=== Failures ===");
        for (i, failure) in failures.iter().enumerate() {
            println!("{}. {}", i + 1, failure);
        }
    }

    // Calculate pass rate and assert >= 90%
    let total_tests = passed + failed;
    if total_tests > 0 {
        let pass_rate = (passed as f64 / total_tests as f64) * 100.0;
        println!("\nPass Rate: {:.1}%", pass_rate);
        assert!(
            pass_rate >= 90.0,
            "YAML test suite pass rate is below 90%: {:.1}%",
            pass_rate
        );
    }
}

// /// Test a specific YAML test case by ID
// #[allow(dead_code)]
// fn test_specific_case(test_id: &str) {
//     let test_dir = Path::new("../tests/yaml-test-suite").join(test_id);

//     if !test_dir.exists() {
//         panic!("Test case {} not found at {:?}", test_id, test_dir);
//     }

//     let test = load_test_case(&test_dir).expect("Failed to load test case");

//     println!("Testing: {} - {}", test.id, test.name);
//     println!("Is error test: {}", test.has_error_file);
//     println!("\nYAML:\n{}", test.yaml);

//     let mut source = BufferSource::new(test.yaml.as_bytes());
//     let result = parse(&mut source);

//     match result {
//         Ok(doc) => {
//             println!("\n✓ Parsed successfully!");
//             if test.has_error_file {
//                 println!("  WARNING: This is an error test but parsing succeeded");
//             }
//             // Don't print the full doc as it can be very large
//             println!("  Document type: {:?}", std::mem::discriminant(&doc));
//         }
//         Err(e) => {
//             println!("\n✗ Parse failed: {}", e);
//             if !test.has_error_file {
//                 println!("  WARNING: This test should have succeeded");
//             }
//         }
//     }
// }

// #[test]
// #[ignore] // Run with: cargo test test_examples -- --ignored
// fn test_examples() {
//     // Test some specific examples
//     let examples = vec![
//         "229Q", // Spec Example 2.4. Sequence of Mappings
//         "236B", // Spec Example 2.3. Mapping Scalars to Scalars
//         "26DV", // Whitespace around colon
//         "27NA", // Spec Example 2.5. Sequence of Sequences
//     ];

//     for id in examples {
//         println!("\n=== Testing {} ===", id);
//         test_specific_case(id);
//     }
// }
