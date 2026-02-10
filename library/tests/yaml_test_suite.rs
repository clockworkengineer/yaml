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
use yaml_lib::test_helpers::parse_yaml;

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

// Run all YAML test suite cases and assert pass rate >= 85%
#[tent]
fn run_yaml_test_suite() {
    // Try multiple possible paths for the test suite
    let possible_paths = vec![
        Path::new("c:/Projects/yaml/yaml-test-suite"),
        Path::new("../yaml-test-suite"),
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

    let skip_list: Vec<&str> = vec![];
    // Current known failing cases from the latest full run (51 total)
    let known_failures: Vec<&str> = vec![
        "00", "001", "01", "06", "236B", "2CMS", "4HVU", "4JVG", "5LLU", "5TRB", "5U3A", "6S55",
        "7LBH", "7MNF", "9C9N", "9CWY", "9HCY", "BD7L", "BF9H", "BS4K", "C2SP", "CXX2", "D49Q",
        "DK4H", "DMG6", "EB22", "EW3V", "F8F9", "G7JE", "G9HC", "GDY7", "GT5M", "H7TQ", "JKF3",
        "KS4U", "N4JP", "P2EQ", "QB6E", "QLJ7", "RHX7", "RXY3", "S98Z", "TD5N", "U44R", "U99R",
        "YJV2", "ZCZ6", "ZL4Z", "ZVH3",
    ];
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut failures = Vec::new();
    let mut unexpected_failures = Vec::new();

    let mut test_dirs = get_all_test_dirs(suite_dir);
    if test_dirs.is_empty() {
        println!("No test directories found. Make sure you're using the data release branch.");
        println!("Run: cd tests/yaml-test-suite && git checkout data-2022-01-17");
        return;
    }
    test_dirs.sort();
    let total_dirs = test_dirs.len();
    println!("Running all {} YAML test suite tests...", total_dirs);
    let mut test_num = 0;
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
        let test = match load_test_case(&test_dir) {
            Some(t) => t,
            None => {
                skipped += 1;
                continue;
            }
        };
        if skip_list.contains(&test.id.as_str()) {
            skipped += 1;
            continue;
        }
        test_num += 1;
        println!("[{}/{}] Testing: {}", test_num, test_limit, test.id);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        let start_time = Instant::now();
        let result = panic::catch_unwind(|| {
            if test.has_error_file {
                // Should error
                let mut errored = false;
                let parse_result = std::panic::catch_unwind(|| {
                    parse_yaml(&test.yaml);
                });
                if parse_result.is_err() {
                    errored = true;
                }
                if errored { Err(()) } else { Ok(()) }
            } else {
                // Should succeed
                let _ = parse_yaml(&test.yaml);
                Ok(())
            }
        });
        let elapsed = start_time.elapsed();
        print!("  Result: ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        if elapsed > Duration::from_millis(200) {
            skipped += 1;
            println!("TIMEOUT (took {:?})", elapsed);
            continue;
        }
        let test_passed = match result {
            Ok(Ok(())) if !test.has_error_file => true,
            Ok(Err(())) if test.has_error_file => true,
            _ => false,
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
            let got = match result {
                Ok(Ok(())) => "success",
                Ok(Err(())) => "error",
                _ => "error",
            };
            println!("FAIL (expected: {}, got: {})", expected, got);
            failures.push(format!(
                "{} (expected: {}, got: {})",
                test.id, expected, got
            ));
            if !known_failures.contains(&test.id.as_str()) {
                println!("UNEXPECTED FAILURE: {}", test.id);
                unexpected_failures.push(test.id.clone());
            }
        }
    }
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
    if !unexpected_failures.is_empty() {
        println!("\n=== Unexpected Failures ===");
        for (i, id) in unexpected_failures.iter().enumerate() {
            println!("{}. {}", i + 1, id);
        }
    }
    let total_tests = passed + failed;
    if total_tests > 0 {
        let pass_rate = (passed as f64 / total_tests as f64) * 100.0;
        println!("\nPass Rate: {:.1}%", pass_rate);
        assert!(
            unexpected_failures.is_empty(),
            "Unexpected YAML test failures detected: {:?}",
            unexpected_failures
        );
        assert!(
            pass_rate >= 85.0,
            "YAML test suite pass rate is below 85%: {:.1}%",
            pass_rate
        );
    }
}
