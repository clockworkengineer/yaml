// Original integration test modules
mod bencode_tests;
mod json_tests;
mod toml_tests;

// New organized integration test modules (split from parser_stringify_integration)
mod anchor_tests;
mod basic_parsing_tests;
mod comma_validation_tests;
mod directive_tests;
mod document_marker_validation_tests;
mod document_structure_tests;
mod error_handling_tests;
mod file_parsing_tests;
mod flow_sequence_key_test;
mod inline_flow_tests;
mod nested_structure_tests;
mod set_tests;
mod tag_coercion_tests;
mod validation_tests;

// Embedded systems integration tests
#[cfg(feature = "embedded")]
mod embedded_tests;

// Flow collection edge case tests
mod debug_flow_hang;
mod flow_trailing_comma_tests;
mod flow_debug;

// Official YAML test suite failing cases
mod official_suite_fixes;
