// =====================================================================================
//  File: mod.rs
//  Location: library/src/integration_tests/
// -------------------------------------------------------------------------------------
//  Purpose:
//      Module declaration file for integration tests in the yaml_lib crate.
//      This file organizes and includes all integration test modules, ensuring
//      comprehensive test coverage for YAML parsing, serialization, and validation.
//
//  Context:
//      - Part of the yaml_lib project, a Rust YAML parser/serializer.
//      - Centralizes integration test module imports for maintainability and clarity.
//      - Supports both standard and embedded system test configurations.
//
// -------------------------------------------------------------------------------------
//  Module Coverage:
//      - Parsing, serialization, and error handling tests
//      - Format-specific tests (JSON, TOML, Bencode)
//      - Edge cases, validation, and embedded system scenarios
// =====================================================================================

// Original integration test modules
mod bencode_tests;
mod json_tests;
mod toml_tests;

// New organized integration test modules (split from parser_stringify_integration)
mod anchor_tests;
mod basic_parsing_tests;
mod block_tag_tests;
mod comma_validation_tests;
mod directive_tests;
mod directive_validation_tests;
mod document_marker_validation_tests;
mod document_structure_tests;
mod error_handling_tests;
mod file_parsing_tests;
mod flow_debug;
mod flow_sequence_key_test;
mod flow_trailing_comma_tests;
mod inline_flow_tests;
mod nested_structure_tests;
mod sequence_null_vs_empty_string_tests;
mod set_tests;
mod tag_coercion_tests;
mod validation_tests;

// Embedded systems integration tests
#[cfg(feature = "embedded")]
mod embedded_tests;

// Official YAML test suite failing cases
mod official_suite_tests;
