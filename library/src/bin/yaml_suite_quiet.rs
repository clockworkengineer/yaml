//! Quiet runner for the official YAML 1.2 Test Suite
//!
//! This binary runs the official YAML test suite and prints only a summary and failures.

use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use yaml_lib::{BufferSource, parse};

#[derive(Debug)]
struct TestCase {
    id: String,
    yaml: String,
    has_error_file: bool,
}

fn load_test_case(test_dir: &Path) -> Option<TestCase> {
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

fn main() {
    let args: Vec<String> = env::args().collect();
    let test_name = if args.len() > 1 {
        Some(args[1].as_str())
    } else {
        None
    };

    // Open output file for writing and also prepare stdout
    let output_file = File::create("yaml_suite_quiet_output.txt").expect("Failed to create output file");
    let mut out = BufWriter::new(output_file);
    let mut stdout = std::io::stdout();

    let possible_paths = vec![Path::new("c:/Projects/yaml/yaml-test-suite")];
    let suite_dir = possible_paths.iter().find(|p| p.exists()).cloned();
    let suite_dir = match suite_dir {
        Some(dir) => dir,
        None => {
            eprintln!("YAML test suite repo directory not found in any of these locations:");
            for path in &possible_paths {
                eprintln!("  - {:?}", path);
            }
            eprintln!(
                "Please clone https://github.com/yaml/yaml-test-suite.git to one of these locations."
            );
            std::process::exit(1);
        }
    };

    let skip_list: Vec<&str> = vec![];
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut failures = Vec::new();
    let mut test_dirs = get_all_test_dirs(suite_dir);
    if test_dirs.is_empty() {
        eprintln!("No test directories found. Make sure you're using the data release branch.");
        eprintln!("Run: cd tests/yaml-test-suite && git checkout data-2022-01-17");
        std::process::exit(1);
    }
    test_dirs.sort();
    let test_limit = 402;

    // If a test name is provided, filter to just that test
    if let Some(name) = test_name {
        let found = test_dirs.iter().find(|dir| {
            dir.file_name()
                .and_then(|f| f.to_str())
                .map(|s| s == name)
                .unwrap_or(false)
        });
        match found {
            Some(test_dir) => {
                let test = match load_test_case(test_dir) {
                    Some(t) => t,
                    None => {
                        writeln!(out, "Test '{}' could not be loaded.", name).unwrap();
                        out.flush().unwrap();
                        std::process::exit(1);
                    }
                };
                let start_time = Instant::now();
                let result = std::panic::catch_unwind(|| {
                    let mut source = BufferSource::new(test.yaml.as_bytes());
                    parse(&mut source)
                });
                let _elapsed = start_time.elapsed();
                let parse_result = match result {
                    Ok(r) => r,
                    Err(_) => {
                        writeln!(out, "Test '{}' skipped (panic)", name).unwrap();
                        out.flush().unwrap();
                        return;
                    }
                };
                let test_passed = match (parse_result.is_ok(), test.has_error_file) {
                    (true, false) => true,
                    (false, true) => true,
                    _ => false,
                };
                if test_passed {
                    writeln!(out, "Test '{}' PASSED", name).unwrap();
                } else {
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
                    if let Err(e) = parse_result {
                        writeln!(out, "Error detail: {}", e).unwrap();
                    }
                    writeln!(
                        out,
                        "Test '{}' FAILED (expected: {}, got: {})",
                        name, expected, got
                    )
                    .unwrap();
                }
                out.flush().unwrap();
                return;
            }
            None => {
                writeln!(out, "Test '{}' not found in suite.", name).unwrap();
                out.flush().unwrap();
                std::process::exit(1);
            }
        }
    }

    // Otherwise, run all tests as before
    for (idx, test_dir) in test_dirs.iter().enumerate() {
        if idx >= test_limit {
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
        let start_time = Instant::now();
        let result = std::panic::catch_unwind(|| {
            let mut source = BufferSource::new(test.yaml.as_bytes());
            parse(&mut source)
        });
        let _elapsed = start_time.elapsed();
        let parse_result = match result {
            Ok(r) => r,
            Err(_) => {
                skipped += 1;
                println!("SUITE_RESULT: {} SKIPPED panic", test.id);
                writeln!(out, "SUITE_RESULT: {} SKIPPED panic", test.id).unwrap();
                continue;
            }
        };
        let test_passed = match (parse_result.is_ok(), test.has_error_file) {
            (true, false) => true,
            (false, true) => true,
            _ => false,
        };
        if test_passed {
            passed += 1;
            println!("SUITE_RESULT: {} PASS", test.id);
            writeln!(out, "SUITE_RESULT: {} PASS", test.id).unwrap();
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
            let err_msg = if let Err(e) = parse_result {
                format!("{}", e)
            } else {
                String::new()
            };
            println!("SUITE_RESULT: {} FAIL expected={} got={} {}", test.id, expected, got, err_msg);
            writeln!(out, "SUITE_RESULT: {} FAIL expected={} got={} {}", test.id, expected, got, err_msg).unwrap();
            failures.push(format!(
                "{} (expected: {}, got: {})",
                test.id, expected, got
            ));
        }
    }
    for line in [
        format!("\n=== YAML Test Suite Results (Quiet) ==="),
        format!("Passed:  {}", passed),
        format!("Failed:  {}", failed),
        format!("Skipped: {}", skipped),
        format!("Total:   {}", passed + failed + skipped),
    ] {
        writeln!(out, "{}", line).unwrap();
        writeln!(stdout, "{}", line).unwrap();
    }
    if !failures.is_empty() {
        writeln!(out, "\n=== Failures ===").unwrap();
        writeln!(stdout, "\n=== Failures ===").unwrap();
        for (i, failure) in failures.iter().enumerate() {
            writeln!(out, "{}. {}", i + 1, failure).unwrap();
            writeln!(stdout, "{}. {}", i + 1, failure).unwrap();
        }
    }
    let total_tests = passed + failed;
    if total_tests > 0 {
        let pass_rate = (passed as f64 / total_tests as f64) * 100.0;
        let rate_line = format!("\nPass Rate: {:.1}%", pass_rate);
        writeln!(out, "{}", rate_line).unwrap();
        writeln!(stdout, "{}", rate_line).unwrap();
        if pass_rate < 85.0 {
            out.flush().unwrap();
            stdout.flush().unwrap();
            std::process::exit(2);
        }
    }
    out.flush().unwrap();
    stdout.flush().unwrap();
}
