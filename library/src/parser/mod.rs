
//! YAML Parser Module
//!
//! Aggregates core modules for YAML parsing, including document parsing, configuration,
//! directive handling, and lexing/tokenization. Provides the common result type for parser operations.
//!
//! Copyright (c) 2026 YAML Library Developers

/// Common result type for parser operations using the library-wide YamlError.
///
/// This is an internal alias to keep signatures concise while
/// gradually migrating away from `Result<T, String>` in parser
/// implementation code.
pub type ParseResult<T> = crate::error::Result<T>;

/// Module for parsing YAML documents
#[path = "document/mod.rs"]
/// document
pub mod document;

/// Parser configuration and builder
pub mod config;

/// Directive parsing (%YAML, %TAG)
pub mod directives;

/// Lexer/Tokenizer for YAML
pub mod lexer;

/// Token stream wrapper for parser integration
pub mod token_stream;

/// Shared parser utilities
#[macro_use]
pub mod utils;
