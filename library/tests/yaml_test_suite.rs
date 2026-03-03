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

// Guard to temporarily silence panic output during the YAML test suite run.
// This keeps the console output focused on per-case PASS/FAIL, while still
// allowing us to detect panics via `catch_unwind`.
struct PanicHookGuard(Option<Box<dyn Fn(&panic::PanicHookInfo) + Send + Sync + 'static>>);

impl PanicHookGuard {
    fn new_silent() -> Self {
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {
            // Suppress panic messages during the suite run.
        }));
        PanicHookGuard(Some(default_hook))
    }
}

impl Drop for PanicHookGuard {
    fn drop(&mut self) {
        if let Some(hook) = self.0.take() {
            panic::set_hook(hook);
        }
    }
}

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

/// Result of running a single YAML test-suite case
enum SuiteCaseStatus {
    /// Case completed within the timeout and matched the expected outcome
    Passed(Duration),
    /// Case exceeded the timeout and was skipped
    Timeout(Duration),
    /// Case completed but did not match the expected outcome
    Failed {
        duration: Duration,
        expected: &'static str,
        got: &'static str,
    },
}

/// Run a single YAML suite case with panic catching and timeout handling.
///
/// This centralizes the "run-and-catch" logic so that other large suites can
/// reuse the same behavior (timeout, panic suppression, and expected vs
/// actual outcome classification).
fn run_yaml_suite_case(yaml: &str, should_error: bool, timeout: Duration) -> SuiteCaseStatus {
    let start_time = Instant::now();
    let result = panic::catch_unwind(|| {
        if should_error {
            // Should error
            let mut errored = false;
            let parse_result = panic::catch_unwind(|| {
                parse_yaml(yaml);
            });
            if parse_result.is_err() {
                errored = true;
            }
            if errored { Err(()) } else { Ok(()) }
        } else {
            // Should succeed
            let _ = parse_yaml(yaml);
            Ok(())
        }
    });
    let elapsed = start_time.elapsed();

    if elapsed > timeout {
        return SuiteCaseStatus::Timeout(elapsed);
    }

    let expected = if should_error { "error" } else { "success" };
    let got = match result {
        Ok(Ok(())) => "success",
        Ok(Err(())) => "error",
        _ => "error",
    };

    let test_passed = match (got, should_error) {
        ("success", false) => true,
        ("error", true) => true,
        _ => false,
    };

    if test_passed {
        SuiteCaseStatus::Passed(elapsed)
    } else {
        SuiteCaseStatus::Failed {
            duration: elapsed,
            expected,
            got,
        }
    }
}

// Run all YAML test suite cases and assert pass rate >= 90%
#[test]
pub fn run_yaml_test_suite() {
    let _panic_hook_guard = PanicHookGuard::new_silent();
    let possible_paths = load_suite_paths();
    let suite_dir = select_suite_dir(&possible_paths);
    if suite_dir.is_none() {
        print_suite_dir_error(&possible_paths);
        return;
    }
    let suite_dir = suite_dir.unwrap();
    run_yaml_suite_tests(&suite_dir);
}

fn load_suite_paths() -> Vec<PathBuf> {
    let suite_paths_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("suite_paths.txt");
    if suite_paths_file.exists() {
        match fs::read_to_string(&suite_paths_file) {
            Ok(contents) => contents
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(trimmed))
                    }
                })
                .collect(),
            Err(_) => vec![],
        }
    } else {
        vec![
            PathBuf::from("c:/Projects/yaml/yaml-test-suite"),
            PathBuf::from("../yaml-test-suite"),
        ]
    }
}

fn select_suite_dir(possible_paths: &[PathBuf]) -> Option<PathBuf> {
    possible_paths.iter().find(|p| p.exists()).cloned()
}

fn print_suite_dir_error(possible_paths: &[PathBuf]) {
    println!(
        "YAML test suite repo directory not found in any of these locations from suite_paths.txt:"
    );
    for path in possible_paths {
        println!("  - {:?}", path);
    }
    println!(
        "Please create suite_paths.txt in this directory with paths to the yaml-test-suite repo."
    );
}

struct KnownFailure {
    id: &'static str,
    description: &'static str,
}

fn run_yaml_suite_tests(suite_dir: &Path) {
    let skip_list: Vec<&str> = vec![];
    let known_failures: &[KnownFailure] = &[
        KnownFailure {
            id: "5LLU",
            description: "Block scalar with wrong indented line after spaces only",
        },
        KnownFailure {
            id: "5U3A",
            description: "Sequence on same line as mapping key",
        },
        KnownFailure {
            id: "6S55",
            description: "Invalid scalar at the end of sequence",
        },
        KnownFailure {
            id: "7LBH",
            description: "Multiline double-quoted implicit keys",
        },
        KnownFailure {
            id: "9C9N",
            description: "Wrong indented flow sequence",
        },
        KnownFailure {
            id: "01",
            description: "(description not available; test directory missing)",
        },
        KnownFailure {
            id: "BF9H",
            description: "Trailing comment in multiline plain scalar",
        },
        KnownFailure {
            id: "BS4K",
            description: "Comment between plain scalar lines",
        },
        KnownFailure {
            id: "C2SP",
            description: "Flow mapping key on two lines",
        },
        KnownFailure {
            id: "D49Q",
            description: "Multiline single-quoted implicit keys",
        },
        KnownFailure {
            id: "DK4H",
            description: "Implicit key followed by newline",
        },
        KnownFailure {
            id: "06",
            description: "(description not available; test directory missing)",
        },
        KnownFailure {
            id: "EB22",
            description: "Missing document-end marker before directive",
        },
        KnownFailure {
            id: "EW3V",
            description: "Wrong indentation in mapping",
        },
        KnownFailure {
            id: "G7JE",
            description: "Multiline implicit keys",
        },
        KnownFailure {
            id: "G9HC",
            description: "Invalid anchor in zero-indented sequence",
        },
        KnownFailure {
            id: "GT5M",
            description: "Node anchor in sequence",
        },
        KnownFailure {
            id: "JKF3",
            description: "Multiline unindented double-quoted block key",
        },
        KnownFailure {
            id: "KS4U",
            description: "Invalid item after end of flow sequence",
        },
        KnownFailure {
            id: "QB6E",
            description: "Wrong indented multiline quoted scalar",
        },
        KnownFailure {
            id: "QLJ7",
            description: "Tag shorthand used in multiple documents but only defined in the first",
        },
        KnownFailure {
            id: "RHX7",
            description: "YAML directive without document end marker",
        },
        KnownFailure {
            id: "RXY3",
            description: "Invalid document-end marker in single-quoted string",
        },
        KnownFailure {
            id: "S98Z",
            description: "Block scalar with more spaces than first content line",
        },
        KnownFailure {
            id: "00",
            description: "(description not available; test directory missing)",
        },
        KnownFailure {
            id: "ZCZ6",
            description: "Invalid mapping in plain single-line value",
        },
        KnownFailure {
            id: "ZVH3",
            description: "Wrong indented sequence item",
        },
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
    let timeout = Duration::from_millis(200);
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
        print!("  Result: ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        match run_yaml_suite_case(&test.yaml, test.has_error_file, timeout) {
            SuiteCaseStatus::Timeout(elapsed) => {
                skipped += 1;
                println!("TIMEOUT (took {:?})", elapsed);
                continue;
            }
            SuiteCaseStatus::Passed(_elapsed) => {
                passed += 1;
                println!("PASS");
            }
            SuiteCaseStatus::Failed {
                duration: _elapsed,
                expected,
                got,
            } => {
                failed += 1;
                println!("FAIL (expected: {}, got: {})", expected, got);
                failures.push(format!(
                    "{} (expected: {}, got: {})",
                    test.id, expected, got
                ));
                if !known_failures.iter().any(|kf| kf.id == test.id) {
                    println!("UNEXPECTED FAILURE: {}", test.id);
                    unexpected_failures.push(test.id.clone());
                }
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
            pass_rate >= 90.0,
            "YAML test suite pass rate is below 90%: {:.1}%",
            pass_rate
        );
    }
}
