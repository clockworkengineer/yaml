//! YAML_lib - A lightweight, modular YAML toolkit for Rust
//!
//! This library provides a flexible YAML implementation with:
//! - Core Node type for representing YAML structures
//! - Parser to build Node trees from streams
//! - Multiple format serializers (YAML, YAML, XML, Bencode)
//! - File and buffer I/O abstractions
//! - Pretty-printing utilities
//! - Unicode-aware file handling
//!
//! Minimum supported Rust version: 1.88.0

/// Module containing constants for the library
mod constants;
/// Module defining error types and handling for YAML operations.
mod error;
/// Module for detecting and handling different Unicode text file formats
mod file;
/// Module providing input/output operations for reading and writing YAML data
mod io;
/// Module containing utility functions and helpers for YAML processing
mod misc;
/// Module containing YAML data structure definitions and node types
mod nodes;
/// Module implementing YAML parsing and value extraction
mod parser;
/// Module for converting YAML structures to formatted strings
mod stringify;
/// Module containing tests for the YAML library
mod test;
/// Module containing  utility functions for the YAML library
mod utils;

/// ============
/// YAML_lib API
/// ============

/// This enum represents different Unicode text file formats with their corresponding byte order marks (BOM)
pub use file::file::Format;
/// This function detects the Unicode format of a text file by examining its byte order mark (BOM)
pub use file::file::detect_format;
/// This function reads a text file and returns its content as a String, handling different Unicode formats
pub use file::file::read_file_to_string;
/// This function writes a string to a file in the specified Unicode format
pub use file::file::write_file_from_string;
/// Destination implementation for writing JSON data to a memory buffer
pub use io::destinations::buffer::Buffer as BufferDestination;
/// Destination implementation for writing JSON data to a file
pub use io::destinations::file::File as FileDestination;
/// Source implementation for reading JSON data from a memory buffer
pub use io::sources::buffer::Buffer as BufferSource;
/// Source implementation for reading JSON data from a file
pub use io::sources::file::File as FileSource;
/// Returns the base node of document number n (0-based), reporting any errors.
pub use misc::get_document_base as get_document;
/// Returns the number of documents in a YAML stream represented by the Documents node.
pub use misc::get_number_of_documents;

/// Returns the current version of the YAML library
pub use misc::get_version as version;
/// Core data structure representing a JSON node and numerical node in the parsed tree
pub use nodes::node::Node;
/// Core data structure representing a numeric value node in the parsed tree
pub use nodes::node::Numeric;
/// Helper function to create a Node from any value that can be converted into a Node
pub use nodes::node::make_node;
/// Parses json data into a Node tree structure
pub use parser::document::parse;
/// Converts a Node tree to bencode format
pub use stringify::bencode::stringify as to_bencode;
/// Converts a Node tree back to YAML format
pub use stringify::default::stringify;
/// Converts a Node tree to JSON format
pub use stringify::json::stringify as to_json;
