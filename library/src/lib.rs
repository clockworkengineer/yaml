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
//!
//! ## Feature Flags
//!
//! - `std` (default): Enable standard library support
//! - `alloc` (default): Enable allocation support (required for most features)
//! - `embedded`: Enable embedded systems optimizations and limits
//! - `parse-only`: Only enable parsing, disable serialization
//! - `stringify`: Enable YAML stringification (requires `alloc`)
//! - `format-converters`: Enable JSON, XML, TOML, Bencode converters
//! - `file-io`: Enable file I/O operations (requires `std`)

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

/// Module containing constants for the library
mod constants;
/// Module for embedded systems support
#[cfg(any(feature = "embedded", doc))]
pub mod embedded;
/// Module defining error types and handling for YAML operations.
pub mod error;
/// Module for detecting and handling different Unicode text file formats
#[cfg(feature = "file-io")]
mod file;
/// Module containing tests for the YAML library
#[cfg(test)]
mod integration_tests;
/// Module providing input/output operations for reading and writing YAML data
mod io;
/// Module containing utility functions and helpers for YAML processing
mod misc;
/// Module containing YAML data structure definitions and node types
mod nodes;
/// Module implementing YAML parsing and value extraction
mod parser;
/// Module for converting YAML structures to formatted strings
#[cfg(feature = "stringify")]
mod stringify;
/// Module containing  utility functions for the YAML library
mod utils;

/// ============
/// YAML_lib API
/// ============
// Error handling types
pub use error::ErrorKind;
pub use error::YamlError;
/// Enhanced error with suggestions and context
#[cfg(feature = "alloc")]
pub use error::enhanced::{EnhancedError, ErrorCode, ErrorSuggestion, Span, SuggestionBuilder};
/// Error recovery strategies and collection
#[cfg(feature = "alloc")]
pub use error::recovery::{
    ErrorCollection, ParserState, RecoveryContext, RecoveryHandler, RecoveryStrategy,
};

/// This enum represents different Unicode text file formats with their corresponding byte order marks (BOM)
#[cfg(feature = "file-io")]
pub use file::file::Format;
/// This function detects the Unicode format of a text file by examining its byte order mark (BOM)
#[cfg(feature = "file-io")]
pub use file::file::detect_format;
/// This function reads a text file and returns its content as a String, handling different Unicode formats
#[cfg(feature = "file-io")]
pub use file::file::read_file_to_string;
/// This function writes a string to a file in the specified Unicode format
#[cfg(feature = "file-io")]
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
/// Fluent builder for constructing Array nodes
#[cfg(feature = "alloc")]
pub use nodes::node::ArrayBuilder;
/// Block style for string nodes
pub use nodes::node::BlockStyle;
/// Fluent builder for constructing Mapping nodes
#[cfg(feature = "alloc")]
pub use nodes::node::MappingBuilder;
/// Core data structure representing a json node and numerical node in the parsed tree
pub use nodes::node::Node;
/// Core data structure representing a numeric value node in the parsed tree
pub use nodes::node::Numeric;
/// Quote type for string nodes
pub use nodes::node::QuoteType;
/// Fluent builder for constructing Set nodes
#[cfg(feature = "alloc")]
pub use nodes::node::SetBuilder;
/// Helper function to create a Node from any value that can be converted into a Node
pub use nodes::node::make_node;
/// Helper function to create a Set node from a vector, ensuring uniqueness
pub use nodes::node::make_set;
/// Parser configuration with builder pattern
pub use parser::config::ParserConfig;
/// Parser configuration builder
pub use parser::config::ParserConfigBuilder;
/// Parses json data into a Node tree structure
pub use parser::document::parse;
/// Converts a Node tree to bencode format
#[cfg(feature = "format-converters")]
pub use stringify::bencode::stringify as to_bencode;
/// Converts a Node tree back to YAML format
#[cfg(feature = "stringify")]
pub use stringify::default::stringify;
/// Converts a Node tree to JSON format
#[cfg(feature = "format-converters")]
pub use stringify::json::stringify as to_json;
#[cfg(feature = "format-converters")]
pub use stringify::json::stringify_pretty as to_json_pretty;
/// Converts a Node tree to TOML format
#[cfg(feature = "format-converters")]
pub use stringify::toml::stringify as to_toml;
#[cfg(feature = "format-converters")]
pub use stringify::toml::stringify_pretty as to_toml_pretty;
/// Converts a Node tree to XML format
#[cfg(feature = "format-converters")]
pub use stringify::xml::stringify as to_xml;
#[cfg(feature = "format-converters")]
pub use stringify::xml::stringify_pretty as to_xml_pretty;
/// Capacity hints for optimizing allocations during parsing
#[cfg(feature = "alloc")]
pub use utils::optimization::CapacityHints;
/// Fast path detector for common YAML patterns
#[cfg(feature = "alloc")]
pub use utils::optimization::FastPathDetector;
/// Lazy tag that defers type coercion until accessed
#[cfg(feature = "alloc")]
pub use utils::optimization::LazyTag;
/// Memory-efficient node builder that reuses allocations
#[cfg(feature = "alloc")]
pub use utils::optimization::NodeBuilder;
/// Performance optimizer combining multiple optimization strategies
#[cfg(feature = "alloc")]
pub use utils::optimization::PerformanceOptimizer;
/// String pool for deduplicating common strings during parsing
#[cfg(all(feature = "std", feature = "alloc"))]
pub use utils::optimization::StringPool;
/// Zero-copy string wrapper
#[cfg(feature = "alloc")]
pub use utils::optimization::ZeroCopyStr;
/// Document statistics for performance analysis
pub use utils::performance::DocumentStats;
/// Performance profiler for measuring multiple operations
#[cfg(all(feature = "std", feature = "alloc"))]
pub use utils::performance::Profiler;
/// Simple timer for measuring operation duration
#[cfg(feature = "std")]
pub use utils::performance::Timer;
/// Utility to compare performance of different approaches
#[cfg(feature = "std")]
pub use utils::performance::compare_performance;
/// Common pre-interned strings for typical YAML keys
#[cfg(feature = "alloc")]
pub use utils::string_interner::CommonStrings;
/// Reference-counted interned string for memory deduplication
#[cfg(feature = "alloc")]
pub use utils::string_interner::InternedString;
/// Statistics about string interning performance
#[cfg(feature = "alloc")]
pub use utils::string_interner::InternerStats;
/// Simple single-threaded string interner
#[cfg(feature = "alloc")]
pub use utils::string_interner::SimpleInterner;
/// Thread-safe string interner with read-write lock
#[cfg(feature = "alloc")]
pub use utils::string_interner::StringInterner;
