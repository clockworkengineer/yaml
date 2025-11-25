//! Module: parser/document/error_builder.rs
//!
//! Provides centralized, consistent error message construction for parsing errors.
//! This ensures all parsing errors have uniform formatting and include relevant context.

// Allow dead code for infrastructure not yet fully utilized
#![allow(dead_code)]

use crate::io::traits::ISource;

/// Category of parsing error for consistent message formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Syntax errors (unexpected characters, malformed structures)
    Syntax,
    /// Indentation and whitespace errors
    Indentation,
    /// Resource limit errors (too many items, infinite loops)
    ResourceLimit,
    /// Structural errors (missing required elements, invalid nesting)
    Structure,
    /// EOF errors (unexpected end of input)
    UnexpectedEof,
}

/// Builder for constructing detailed, consistent error messages.
///
/// Provides a fluent API for building error messages with context information
/// from the source stream.
///
/// # Example
///
/// ```ignore
/// ErrorBuilder::new(ErrorCategory::Syntax)
///     .message("Expected ':' in mapping")
///     .context(source)
///     .build()
/// ```
pub struct ErrorBuilder {
    category: ErrorCategory,
    message: String,
    source_context: Option<SourceContext>,
    hint: Option<String>,
}

/// Context information from the source stream
struct SourceContext {
    current_char: String,
    indent_level: usize,
}

impl ErrorBuilder {
    /// Creates a new error builder with the specified category
    #[allow(dead_code)]
    pub fn new(category: ErrorCategory) -> Self {
        Self {
            category,
            message: String::new(),
            source_context: None,
            hint: None,
        }
    }

    /// Sets the main error message
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }

    /// Adds source context (current character and indent level)
    pub fn context(mut self, source: &mut dyn ISource) -> Self {
        let current_char = match source.current() {
            Some(c) => c.to_string(),
            None => "EOF".to_string(),
        };
        self.source_context = Some(SourceContext {
            current_char,
            indent_level: source.get_current_indent_level(),
        });
        self
    }

    /// Adds a hint for how to fix the error
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Builds the final error string
    pub fn build(self) -> String {
        let mut result = self.message;

        if let Some(ctx) = self.source_context {
            result.push_str(&format!(
                " (current: '{}', indent: {})",
                ctx.current_char, ctx.indent_level
            ));
        }

        if let Some(hint) = self.hint {
            result.push_str(&format!(" [Hint: {}]", hint));
        }

        result
    }
}

/// Convenience functions for common error types

/// Creates a syntax error with source context
pub fn syntax_error(source: &mut dyn ISource, message: &str) -> String {
    ErrorBuilder::new(ErrorCategory::Syntax)
        .message(message)
        .context(source)
        .build()
}

/// Creates an indentation error with source context
pub fn indentation_error(source: &mut dyn ISource, message: &str) -> String {
    ErrorBuilder::new(ErrorCategory::Indentation)
        .message(message)
        .context(source)
        .build()
}

/// Creates a resource limit error (no source context needed)
pub fn limit_error(context: &str, max_value: usize, limit_type: &str) -> String {
    format!(
        "{} exceeded maximum {} ({}) - possible infinite loop",
        context, limit_type, max_value
    )
}

/// Creates a structure error with source context
pub fn structure_error(source: &mut dyn ISource, message: &str) -> String {
    ErrorBuilder::new(ErrorCategory::Structure)
        .message(message)
        .context(source)
        .build()
}

/// Creates an EOF error
pub fn eof_error(context: &str) -> String {
    format!("Unexpected end of input in {}", context)
}

/// Creates an EOF error with expected element
#[allow(dead_code)]
pub fn eof_expecting(expected: &str) -> String {
    format!("Unexpected end of input while expecting {}", expected)
}

/// Creates an "expected X" error with source context
#[allow(dead_code)]
pub fn expected_error(source: &mut dyn ISource, expected: &str) -> String {
    ErrorBuilder::new(ErrorCategory::Syntax)
        .message(format!("Expected {}", expected))
        .context(source)
        .build()
}

/// Creates an "unexpected X" error with source context
#[allow(dead_code)]
pub fn unexpected_error(source: &mut dyn ISource, found: &str) -> String {
    ErrorBuilder::new(ErrorCategory::Syntax)
        .message(format!("Unexpected {}", found))
        .context(source)
        .build()
}

/// Creates an empty anchor/alias error
#[allow(dead_code)]
pub fn empty_name_error(name_type: &str) -> String {
    format!("Empty {} name", name_type)
}

/// Creates an undefined reference error
#[allow(dead_code)]
pub fn undefined_reference(ref_type: &str, name: &str) -> String {
    format!("Undefined {} reference: {}", ref_type, name)
}

/// Creates a duplicate definition error
#[allow(dead_code)]
pub fn duplicate_error(item_type: &str, name: &str) -> String {
    format!("Duplicate {}: {}", item_type, name)
}

/// Creates an inconsistent indentation error with details
#[allow(dead_code)]
pub fn inconsistent_indent_error(
    source: &mut dyn ISource,
    expected: usize,
    got: usize,
    context: &str,
) -> String {
    ErrorBuilder::new(ErrorCategory::Indentation)
        .message(format!(
            "Inconsistent indentation in {}: expected {}, got {}",
            context, expected, got
        ))
        .context(source)
        .build()
}

/// Creates a validation error for forbidden characters/patterns
pub fn forbidden_error(source: &mut dyn ISource, what: &str, where_forbidden: &str) -> String {
    ErrorBuilder::new(ErrorCategory::Syntax)
        .message(format!("{} are not allowed {}", what, where_forbidden))
        .context(source)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_syntax_error() {
        let mut source = Buffer::new(b"test");
        let error = syntax_error(&mut source, "Invalid character");
        assert!(error.contains("Invalid character"));
        assert!(error.contains("current:"));
        assert!(error.contains("indent:"));
    }

    #[test]
    fn test_indentation_error() {
        let mut source = Buffer::new(b"  test");
        let error = indentation_error(&mut source, "Wrong indent");
        assert!(error.contains("Wrong indent"));
        assert!(error.contains("indent:")); // Just check for indent field, not specific value
        assert!(error.contains("current:"));
    }

    #[test]
    fn test_limit_error() {
        let error = limit_error("Sequence parsing", 100000, "loop iterations");
        assert!(error.contains("Sequence parsing"));
        assert!(error.contains("100000"));
        assert!(error.contains("loop iterations"));
        assert!(error.contains("infinite loop"));
    }

    #[test]
    fn test_structure_error() {
        let mut source = Buffer::new(b":");
        let error = structure_error(&mut source, "Missing key");
        assert!(error.contains("Missing key"));
    }

    #[test]
    fn test_eof_error() {
        let error = eof_error("inline mapping");
        assert_eq!(error, "Unexpected end of input in inline mapping");
    }

    #[test]
    fn test_eof_expecting() {
        let error = eof_expecting("closing bracket");
        assert_eq!(error, "Unexpected end of input while expecting closing bracket");
    }

    #[test]
    fn test_expected_error() {
        let mut source = Buffer::new(b",");
        let error = expected_error(&mut source, "':'");
        assert!(error.contains("Expected ':'"));
        assert!(error.contains("current: ','"));
    }

    #[test]
    fn test_unexpected_error() {
        let mut source = Buffer::new(b"#");
        let error = unexpected_error(&mut source, "comment");
        assert!(error.contains("Unexpected comment"));
    }

    #[test]
    fn test_empty_name_error() {
        let error = empty_name_error("anchor");
        assert_eq!(error, "Empty anchor name");
    }

    #[test]
    fn test_undefined_reference() {
        let error = undefined_reference("anchor", "myanchor");
        assert_eq!(error, "Undefined anchor reference: myanchor");
    }

    #[test]
    fn test_duplicate_error() {
        let error = duplicate_error("anchor", "myanchor");
        assert_eq!(error, "Duplicate anchor: myanchor");
    }

    #[test]
    fn test_inconsistent_indent_error() {
        let mut source = Buffer::new(b"test");
        let error = inconsistent_indent_error(&mut source, 2, 4, "sequence");
        assert!(error.contains("Inconsistent indentation in sequence"));
        assert!(error.contains("expected 2"));
        assert!(error.contains("got 4"));
    }

    #[test]
    fn test_forbidden_error() {
        let mut source = Buffer::new(b"\t");
        let error = forbidden_error(&mut source, "Tabs", "as indentation in YAML");
        assert!(error.contains("Tabs are not allowed as indentation in YAML"));
    }

    #[test]
    fn test_error_builder_with_hint() {
        let mut source = Buffer::new(b":");
        let error = ErrorBuilder::new(ErrorCategory::Syntax)
            .message("Missing key before colon")
            .context(&mut source)
            .hint("Add a key before the ':'")
            .build();

        assert!(error.contains("Missing key before colon"));
        assert!(error.contains("Hint:"));
        assert!(error.contains("Add a key"));
    }

    #[test]
    fn test_error_builder_at_eof() {
        let mut source = Buffer::new(b"");
        let error = syntax_error(&mut source, "Unexpected end");
        assert!(error.contains("current: 'EOF'"));
        assert!(error.contains("indent: 0"));
    }
}
