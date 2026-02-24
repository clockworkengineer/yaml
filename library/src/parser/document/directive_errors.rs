
//! Directive Error Helpers
//!
//! Centralizes error construction for YAML directive parsing, providing helpers
//! for consistent error messages and easier future maintenance.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::error::{ErrorKind, YamlError};

/// Centralized constructors for directive-related errors and messages.
///
/// Behavior-neutral: all strings exactly match existing messages.
pub struct DirectiveErrors;

impl DirectiveErrors {
    /// Duplicate %YAML directive encountered in the same document.
    pub fn duplicate_yaml_directive() -> YamlError {
        YamlError::new(ErrorKind::ParseError, "Duplicate YAML directive")
    }

    /// Missing version after %YAML.
    pub fn missing_yaml_version() -> YamlError {
        YamlError::new(
            ErrorKind::ParseError,
            "Missing YAML version after %YAML directive",
        )
    }

    /// Invalid YAML major when parsing version components generically.
    pub fn invalid_yaml_major_version_generic() -> YamlError {
        YamlError::new(ErrorKind::ParseError, "Invalid YAML major version")
    }

    /// Invalid YAML minor when parsing version components generically.
    pub fn invalid_yaml_minor_version_generic() -> YamlError {
        YamlError::new(ErrorKind::ParseError, "Invalid YAML minor version")
    }

    /// Invalid YAML major version with number formatting.
    pub fn invalid_yaml_major_version_num(major: u8) -> YamlError {
        YamlError::new(
            ErrorKind::ParseError,
            alloc::format!("Invalid YAML major version: {}", major),
        )
    }

    /// Malformed %TAG directive format.
    pub fn malformed_tag_directive() -> YamlError {
        YamlError::new(
            ErrorKind::ParseError,
            "YAML compliance error: Malformed %TAG directive. Expected format: %TAG <handle> <prefix>",
        )
    }

    /// Undefined explicit tag handle used without a %TAG directive.
    pub fn undefined_tag_handle(handle: &str) -> YamlError {
        YamlError::new(
            ErrorKind::ParseError,
            alloc::format!(
                "Undefined tag handle '{}'. Define it with a %TAG directive",
                handle
            ),
        )
    }

    /// Message: directive must be followed by a document.
    /// Returns the message string to be used with token-based error builders.
    pub fn must_be_followed_by_document_msg() -> &'static str {
        "Directive must be followed by a document"
    }

    /// Message: mid-stream directives not allowed unless previous document ended with '...'.
    pub fn directives_not_allowed_midstream_msg() -> &'static str {
        "Directives are not allowed after content unless the previous document ended with '...'"
    }
}
