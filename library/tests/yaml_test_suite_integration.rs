// Integration test for the official YAML 1.2 Test Suite
// This file should be placed in the `tests/` directory for proper test harness discovery.

mod yaml_test_suite;

#[test]
fn run_yaml_test_suite() {
    crate::yaml_test_suite::run_yaml_test_suite();
}
