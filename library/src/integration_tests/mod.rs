// Original integration test modules
mod bencode_tests;
mod json_tests;
mod toml_tests;

// New organized integration test modules (split from parser_stringify_integration)
mod anchor_tests;
mod basic_parsing_tests;
mod document_structure_tests;
mod error_handling_tests;
mod file_parsing_tests;
mod inline_flow_tests;
mod nested_structure_tests;
mod set_tests;
mod tag_coercion_tests;

// Embedded systems integration tests
#[cfg(feature = "embedded")]
mod embedded_tests;
