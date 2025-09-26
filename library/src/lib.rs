
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

/// Module providing input/output operations for reading and writing YAML data
pub mod io;
// /// Module containing YAML data structure definitions and node types
pub mod nodes;
// /// Module implementing YAML parsing and value extraction
// pub mod parser;
// /// Module defining error types and handling for YAML operations.
// pub mod error;
// /// Module for converting YAML structures to formatted strings
// pub mod stringify;
// /// Module handling YAML file reading and writing operations
// pub mod file;
// /// Module containing utility functions and helpers for YAML processing
// pub mod misc;
// 
// ///
// /// YAML_lib API
// ///
// 
// /// Returns the current version of the YAML library
// pub use misc::get_version as version;
// /// Strip whitespace from a string.
// pub use misc::strip as strip_whitespace;
// /// Prints a formatted string to the destination.
// pub use misc::print as print;
// /// This enum represents different Unicode text file formats with their corresponding byte order marks (BOM)
// pub use file::file::Format as Format;
// /// This function detects the Unicode format of a text file by examining its byte order mark (BOM)
// pub use file::file::detect_format as detect_format;
// /// This function reads a text file and returns its content as a String, handling different Unicode formats
// pub use file::file::read_file_to_string as read_file_to_string;
// /// This function writes a string to a file in the specified Unicode format
// pub use file::file::write_file_from_string as write_file_from_string;
// 
// /// Source implementation for reading YAML data from a memory buffer
// pub use io::sources::buffer::Buffer as BufferSource;
// /// Destination implementation for writing YAML data to a memory buffer
// pub use io::destinations::buffer::Buffer as BufferDestination;
// /// Source implementation for reading YAML data from a file
// pub use io::sources::file::File as FileSource;
// /// Destination implementation for writing YAML data to a file
// pub use io::destinations::file::File as FileDestination;
// /// Core data structure representing a YAML node and numerical node in the parsed tree
// pub use nodes::node::Node as Node;
// /// Core data structure representing a numeric value node in the parsed tree
// pub use nodes::node::Numeric as Numeric;
// /// Converts a Node tree back to YAML format
// pub use stringify::default::stringify as stringify;
// /// Parses YAML data into a Node tree structure
// pub use parser::default::parse as parse;
// /// Converts a Node tree to YAML format
// pub use stringify::bencode::stringify as to_bencode;
// /// Converts a Node tree to YAML format
// pub use stringify::yaml::stringify as to_yaml;
// /// Converts a Node tree to XML format
// pub use stringify::xml::stringify as to_xml;
// /// Converts a Node tree to TOML format
// pub use stringify::toml::stringify as to_toml;
