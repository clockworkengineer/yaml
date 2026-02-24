/**
 * YAML_lib: Modular YAML Toolkit for Rust
 *
 * This crate provides a lightweight, modular, and flexible YAML implementation for Rust.
 * It includes a core Node type, parser, multiple format serializers (YAML, JSON, XML, TOML, Bencode),
 * file and buffer I/O abstractions, pretty-printing, and Unicode-aware file handling.
 *
 * # Features
 * - Core Node type for YAML structures
 * - Streaming parser
 * - Multiple format serializers
 * - File and buffer I/O
 * - Pretty-printing utilities
 * - Unicode support
 *
 * # Minimum Supported Rust Version
 * 1.88.0
 *
 * # Feature Flags
 * - `std` (default): Enable standard library support
 * - `alloc`: Enable heap-allocated data structures
 * - `serde`: Enable Serde serialization/deserialization
 * - `debug-anchors`: Enable debug logging for anchors
 *
 * # Usage
 * Add `yaml_lib` as a dependency and use the provided modules for YAML parsing, serialization, and processing.
 */
pub use crate::parser::utils::macros::anchors_debug_macro::*;
pub use crate::parser::utils::macros::lexer_log_macro::*;

#[macro_use]
pub mod parser;
/// - File and buffer I/O abstractions
/// - Pretty-printing utilities
/// - Unicode-aware file handling
///
/// Minimum supported Rust version: 1.88.0
///
/// ## Feature Flags
///
/// - `std` (default): Enable standard library support
/// - `alloc` (default): Enable allocation support (required for most features)
/// - `embedded`: Enable embedded systems optimizations and limits
/// - `parse-only`: Only enable parsing, disable serialization
/// - `stringify`: Enable YAML stringification (requires `alloc`)
/// - `format-converters`: Enable JSON, XML, TOML, Bencode converters
/// - `file-io`: Enable file I/O operations (requires `std`)

#[cfg_attr(not(feature = "std"), no_std)]

/// Common test helpers for integration/unit tests
pub mod test_helpers;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Module containing constants for the library
mod constants;
/// Module for developer tools (debugging, inspection, diffing, tracing)
#[cfg(feature = "alloc")]
pub mod devtools;
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
/// Module for converting YAML structures to formatted strings
#[cfg(feature = "stringify")]
mod stringify;
/// Module for testing infrastructure (fuzzing, property testing, safety)
#[cfg(feature = "alloc")]
pub mod testing;
/// Module containing  utility functions for the YAML library
mod utils;
/// Module for YAML validation and schema support
#[cfg(feature = "alloc")]
pub mod validation;

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
/// Destination implementation for writing YAML data to a memory buffer
pub use io::destinations::buffer::Buffer as BufferDestination;
/// Destination implementation for writing YAML data to a file
pub use io::destinations::file::File as FileDestination;
/// Source implementation for reading YAML data from a memory buffer
pub use io::sources::buffer::Buffer as BufferSource;
/// Source implementation for reading YAML data from a file
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
/// Core data structure representing a YAML node in the parsed tree
pub use nodes::node::Node;
/// Core data structure representing a numeric value node in the parsed tree
pub use nodes::node::Numeric;
/// Quote type for string nodes
pub use nodes::node::QuoteType;
/// Fluent builder for constructing Set nodes
#[cfg(feature = "alloc")]
pub use nodes::node::SetBuilder;
/// Helper function to create a Node from any value that can be converted into a Node
pub use nodes::util::make_node;
/// Helper function to create a Set node from a vector, ensuring uniqueness
pub use nodes::util::make_set;
/// Parser configuration with builder pattern
pub use parser::config::ParserConfig;
/// Parser configuration builder
pub use parser::config::ParserConfigBuilder;
/// Parses YAML data into a Node tree structure
pub use parser::document::parse;

/// Internal helper that funnels all public parse_* APIs through the same
/// source-based entry point. This keeps configuration and future recovery
/// wiring centralized.
fn parse_from_source(
    source: &mut dyn crate::io::traits::ISource,
    _config: &ParserConfig,
) -> crate::error::Result<Node> {
    // ParserConfig enforcement is currently handled within the parser
    // implementation; this helper exists so future extensions (limits,
    // recovery, instrumentation) only need to be wired in one place.
    parser::document::parse(source)
}

/// Parse YAML from an in-memory string using the default parser configuration.
pub fn parse_string(yaml: &str) -> crate::error::Result<Node> {
    let mut source = BufferSource::new(yaml.as_bytes());
    let config = ParserConfig::default();
    parse_from_source(&mut source, &config)
}

/// Parse YAML from a file path using the default parser configuration.
#[cfg(feature = "file-io")]
pub fn parse_file(path: &str) -> crate::error::Result<Node> {
    match read_file_to_string(path) {
        Ok(contents) => parse_string(&contents),
        Err(e) => Err(e.to_string().into()),
    }
}

/// Parse YAML from a string with a custom parser configuration.
pub fn parse_with_config(yaml: &str, config: ParserConfig) -> crate::error::Result<Node> {
    let mut source = BufferSource::new(yaml.as_bytes());
    parse_from_source(&mut source, &config)
}

/// Parse YAML from a string while preparing for future error recovery support.
///
/// Currently this behaves like `parse_string`, returning the parsed node and
/// an empty list of secondary errors; the `RecoveryHandler` will be wired into
/// the parser once full recovery support is implemented.
#[cfg(feature = "alloc")]
pub fn parse_string_with_recovery(
    yaml: &str,
    _handler: RecoveryHandler,
) -> crate::error::Result<(Node, Vec<YamlError>)> {
    let node = parse_string(yaml)?;
    Ok((node, Vec::new()))
}
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
/// Iterator for traversing node trees
#[cfg(feature = "alloc")]
pub use utils::streaming::NodeIterator;
/// Extension trait for Node providing iterator methods
#[cfg(feature = "alloc")]
pub use utils::streaming::NodeIteratorExt;
/// Path for accessing nested nodes
#[cfg(feature = "alloc")]
pub use utils::streaming::NodePath;
/// Stream processor for efficient large document handling
#[cfg(feature = "alloc")]
pub use utils::streaming::NodeStream;
/// Path segment (key or index) for node access
#[cfg(feature = "alloc")]
pub use utils::streaming::PathSegment;
/// Traversal order for iterating through nodes
#[cfg(feature = "alloc")]
pub use utils::streaming::TraversalOrder;
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

// Validation API
/// Validation engine and error types
#[cfg(feature = "alloc")]
pub use validation::engine::{SchemaValidator, ValidationContext};
/// Schema types for defining validation rules
#[cfg(feature = "alloc")]
pub use validation::schema::{ArraySchema, ObjectSchema, PropertySchema, Schema, SchemaType};
/// Built-in validators
#[cfg(feature = "alloc")]
pub use validation::validators::{
    CustomValidator, EnumValidator, LengthValidator, PatternValidator, RangeValidator,
    RequiredValidator, TypeValidator, ValidationResult, Validator,
};

/// Custom serializer support
#[cfg(feature = "alloc")]
pub use stringify::serializer::{Serializer, SerializerRegistry, TaggedSerializer, TypeSerializer};
/// Streaming serialization
#[cfg(feature = "alloc")]
pub use stringify::streaming::StreamingSerializer;

// Testing API
/// Fuzzing infrastructure for discovering bugs
#[cfg(feature = "alloc")]
pub use testing::fuzzing::{FuzzResult, FuzzRng, YamlFuzzer, fuzz_parse, fuzz_roundtrip};
/// Property-based testing for checking invariants
#[cfg(feature = "alloc")]
pub use testing::property::{Property, PropertyResult, PropertySuite};
/// Memory safety auditing tools
#[cfg(feature = "alloc")]
pub use testing::safety::{
    MemoryStats, SafetyAudit, SafetyIssue, audit_node, calculate_memory_stats,
};

// Developer Tools API
/// Debugging utilities
#[cfg(feature = "alloc")]
pub use devtools::debug::{DebugAssert, DebugContext, DebugLevel, NodeDebugger};
/// Node diffing and comparison
#[cfg(feature = "alloc")]
pub use devtools::diff::{Diff, DiffResult, DiffType, diff_nodes};
/// Node inspection and introspection
#[cfg(feature = "alloc")]
pub use devtools::inspect::{
    NodeInfo, NodeType, find_by_type, has_anchor, has_tag, node_depth, node_size, node_summary,
    node_type, print_tree,
};
#[cfg(all(feature = "alloc", feature = "std"))]
pub use devtools::trace::TracedTimer;
/// Execution tracing
#[cfg(feature = "alloc")]
pub use devtools::trace::{TraceEntry, TraceEvent, TraceGuard, Tracer};
