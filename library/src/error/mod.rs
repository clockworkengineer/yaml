//! YAML Error Handling Module
//!
//! Aggregates error types, message constants, enhanced error handling, and recovery strategies
//! for YAML parsing and processing. Provides structured error handling with detailed context
//! and unified error conversion for robust diagnostics and reporting.
//!
//! Copyright (c) 2026 YAML Library Developers

/// Allow conversion from ValidationError to YamlError for unified error handling
#[cfg(feature = "alloc")]
impl From<crate::validation::error::ValidationError> for YamlError {
    fn from(e: crate::validation::error::ValidationError) -> Self {
        YamlError::new(ErrorKind::ValidationError, e.to_string())
    }
}
/// Error types for YAML parsing and operations
///
/// This module provides structured error handling with detailed context
/// about where and why errors occurred.

/// Error message constants
pub mod messages;

/// Enhanced error handling with suggestions and recovery
pub mod enhanced;

/// Error recovery strategies
pub mod recovery;

// Re-export key types for convenience
#[cfg(feature = "alloc")]
pub use enhanced::RecoveryStrategy;

#[cfg(feature = "std")]
use std::fmt;

#[cfg(not(feature = "std"))]
use core::fmt;

/// Specific category of error that occurred
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// Syntax error in YAML structure
    SyntaxError,
    /// Parse error (generic parsing failure)
    ParseError,
    /// Unterminated string (missing closing quote)
    UnterminatedString,
    /// Invalid or unknown tag
    InvalidTag,
    /// Reference to undefined anchor/alias
    UndefinedAlias,
    /// Invalid anchor definition
    InvalidAnchor,
    /// Duplicate anchor name
    DuplicateAnchor,
    /// I/O operation failed
    IoError,
    /// Validation error (structure or limits)
    ValidationError,
    /// Unexpected end of input
    UnexpectedEof,
    /// Unexpected character encountered
    UnexpectedCharacter,
    /// Invalid escape sequence
    InvalidEscape,
    /// Unsupported operation or node type
    Unsupported,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::SyntaxError => write!(f, "Syntax error"),
            ErrorKind::ParseError => write!(f, "Parse error"),
            ErrorKind::UnterminatedString => write!(f, "Unterminated string"),
            ErrorKind::InvalidTag => write!(f, "Invalid tag"),
            ErrorKind::UndefinedAlias => write!(f, "Undefined alias"),
            ErrorKind::InvalidAnchor => write!(f, "Invalid anchor"),
            ErrorKind::DuplicateAnchor => write!(f, "Duplicate anchor"),
            ErrorKind::IoError => write!(f, "I/O error"),
            ErrorKind::ValidationError => write!(f, "Validation error"),
            ErrorKind::UnexpectedEof => write!(f, "Unexpected end of input"),
            ErrorKind::UnexpectedCharacter => write!(f, "Unexpected character"),
            ErrorKind::InvalidEscape => write!(f, "Invalid escape sequence"),
            ErrorKind::Unsupported => write!(f, "Unsupported operation"),
        }
    }
}

/// Structured error type with context information
///
/// Provides detailed information about errors including:
/// - The kind/category of error
/// - A descriptive message
/// - Optional line and column information
///
/// # Example
/// ```
/// use yaml_lib::error::{YamlError, ErrorKind};
///
/// let error = YamlError::new(
///     ErrorKind::SyntaxError,
///     "Expected ':' in mapping"
/// ).with_position(5, 12);
///
/// assert_eq!(error.kind(), &ErrorKind::SyntaxError);
/// assert_eq!(error.line(), Some(5));
/// assert_eq!(error.column(), Some(12));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlError {
    kind: ErrorKind,
    message: alloc::string::String,
    line: Option<usize>,
    column: Option<usize>,
}

impl YamlError {
    /// Create a new error with the specified kind and message
    pub fn new(kind: ErrorKind, message: impl Into<alloc::string::String>) -> Self {
        Self {
            kind,
            message: message.into(),
            line: None,
            column: None,
        }
    }

    /// Add line and column position information
    pub fn with_position(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Add line position information
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Get the error kind
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the line number where the error occurred, if available
    pub fn line(&self) -> Option<usize> {
        self.line
    }

    /// Get the column number where the error occurred, if available
    pub fn column(&self) -> Option<usize> {
        self.column
    }
}

impl fmt::Display for YamlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)?;

        if let Some(line) = self.line {
            if let Some(col) = self.column {
                write!(f, " at line {}, column {}", line, col)?;
            } else {
                write!(f, " at line {}", line)?;
            }
        }

        Ok(())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for YamlError {}

/// Convenience type alias for Results using YamlError
pub type Result<T> = core::result::Result<T, YamlError>;

/// Convert from String to YamlError (for backward compatibility)
impl From<alloc::string::String> for YamlError {
    fn from(message: alloc::string::String) -> Self {
        // Try to infer error kind from message content
        let kind = if message.contains("Unterminated") {
            ErrorKind::UnterminatedString
        } else if message.contains("Unexpected end") || message.contains("EOF") {
            ErrorKind::UnexpectedEof
        } else if message.contains("Unexpected character") {
            ErrorKind::UnexpectedCharacter
        } else if message.contains("anchor") {
            if message.contains("Undefined") {
                ErrorKind::UndefinedAlias
            } else if message.contains("Duplicate") {
                ErrorKind::DuplicateAnchor
            } else {
                ErrorKind::InvalidAnchor
            }
        } else if message.contains("tag") {
            ErrorKind::InvalidTag
        } else if message.contains("escape") {
            ErrorKind::InvalidEscape
        } else if message.contains("Validation") {
            ErrorKind::ValidationError
        } else {
            ErrorKind::ParseError
        };

        Self {
            kind,
            message,
            line: None,
            column: None,
        }
    }
}

impl From<&str> for YamlError {
    fn from(message: &str) -> Self {
        alloc::string::String::from(message).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = YamlError::new(ErrorKind::SyntaxError, "test error");
        assert_eq!(error.kind(), &ErrorKind::SyntaxError);
        assert_eq!(error.message(), "test error");
        assert_eq!(error.line(), None);
        assert_eq!(error.column(), None);
    }

    #[test]
    fn test_error_with_position() {
        let error = YamlError::new(ErrorKind::ParseError, "test").with_position(10, 5);
        assert_eq!(error.line(), Some(10));
        assert_eq!(error.column(), Some(5));
    }

    #[test]
    fn test_error_display() {
        let error = YamlError::new(ErrorKind::UnterminatedString, "Missing quote");
        assert_eq!(error.to_string(), "Unterminated string: Missing quote");

        let error_with_pos = error.with_position(5, 12);
        assert_eq!(
            error_with_pos.to_string(),
            "Unterminated string: Missing quote at line 5, column 12"
        );
    }

    #[test]
    fn test_error_from_string() {
        let error: YamlError = "Unterminated string literal".into();
        assert_eq!(error.kind(), &ErrorKind::UnterminatedString);

        let error: YamlError = "Undefined anchor reference: foo".into();
        assert_eq!(error.kind(), &ErrorKind::UndefinedAlias);

        let error: YamlError = "Unexpected end of input".into();
        assert_eq!(error.kind(), &ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_error_kind_display() {
        assert_eq!(ErrorKind::SyntaxError.to_string(), "Syntax error");
        assert_eq!(ErrorKind::InvalidTag.to_string(), "Invalid tag");
        assert_eq!(ErrorKind::ValidationError.to_string(), "Validation error");
    }
}
