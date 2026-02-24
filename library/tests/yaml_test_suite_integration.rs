//! Integration Test: YAML 1.2 Official Test Suite
//!
//! This test runs the official YAML 1.2 Test Suite against the library implementation.
//! Place this file in the `tests/` directory for proper test harness discovery.
//!
//! # Purpose
//! - Ensures compliance with the YAML 1.2 specification
//! - Validates parser correctness and edge case handling
//!
//! # Usage
//! Run with `cargo test` to execute the full YAML test suite integration.

mod yaml_test_suite;

#[test]
fn run_yaml_test_suite() {
    crate::yaml_test_suite::run_yaml_test_suite();
}
