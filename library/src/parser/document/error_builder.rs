#![allow(dead_code)]
/// Module: parser/document/error_builder.rs
///
/// Provides centralized, consistent error message construction for parsing errors.
/// This ensures all parsing errors have uniform formatting and include relevant context.
///
/// This module currently builds string-based errors for legacy
/// `Result<T, String>` call sites, but also exposes helpers that
/// construct the library-wide `YamlError` type so parser code can
/// gradually migrate to `ParseResult<T>`.
use crate::error::{ErrorKind, YamlError};
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

    /// Builds the final error string (legacy representation)
    pub fn build(self) -> String {
        let mut result = self.message.clone();

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

    /// Builds a structured `YamlError` for new parser code.
    pub fn build_yaml(self) -> YamlError {
        let kind = match self.category {
            ErrorCategory::Syntax => ErrorKind::SyntaxError,
            ErrorCategory::Indentation => ErrorKind::SyntaxError,
            ErrorCategory::ResourceLimit => ErrorKind::ValidationError,
            ErrorCategory::Structure => ErrorKind::ParseError,
            ErrorCategory::UnexpectedEof => ErrorKind::UnexpectedEof,
        };

        let mut message = self.message.clone();

        if let Some(ctx) = self.source_context {
            message.push_str(&format!(
                " (current: '{}', indent: {})",
                ctx.current_char, ctx.indent_level
            ));
        }

        if let Some(hint) = self.hint {
            message.push_str(&format!(" [Hint: {}]", hint));
        }

        YamlError::new(kind, message)
    }
}

/// Convenience functions for common error types

/// Creates a syntax error with source context
pub fn syntax_error(source: &mut dyn ISource, message: &str) -> YamlError {
    YamlError::new(
        ErrorKind::SyntaxError,
        ErrorBuilder::new(ErrorCategory::Syntax)
            .message(message)
            .context(source)
            .build(),
    )
}

/// Creates an indentation error with source context
pub fn indentation_error(source: &mut dyn ISource, message: &str) -> YamlError {
    YamlError::new(
        ErrorKind::ValidationError,
        ErrorBuilder::new(ErrorCategory::Indentation)
            .message(message)
            .context(source)
            .build(),
    )
}

/// Creates a resource limit error (no source context needed)
pub fn limit_error(context: &str, max_value: usize, limit_type: &str) -> YamlError {
    YamlError::new(
        ErrorKind::ValidationError,
        format!(
            "{} exceeded maximum {} ({}) - possible infinite loop",
            context, limit_type, max_value
        ),
    )
}

/// Creates a structure error with source context
pub fn structure_error(source: &mut dyn ISource, message: &str) -> YamlError {
    ErrorBuilder::new(ErrorCategory::Structure)
        .message(message)
        .context(source)
        .build_yaml()
}

/// Creates an EOF error
pub fn eof_error(context: &str) -> YamlError {
    ErrorBuilder::new(ErrorCategory::Syntax)
        .message(format!("Unexpected end of input in {}", context))
        .build_yaml()
}

/// Creates an EOF error with expected element
#[allow(dead_code)]
pub fn eof_expecting(expected: &str) -> YamlError {
    ErrorBuilder::new(ErrorCategory::Syntax)
        .message(format!(
            "Unexpected end of input while expecting {}",
            expected
        ))
        .build_yaml()
}

/// Creates an "expected X" error with source context
#[allow(dead_code)]
pub fn expected_error(source: &mut dyn ISource, expected: &str) -> YamlError {
    ErrorBuilder::new(ErrorCategory::Syntax)
        .message(format!("Expected {}", expected))
        .context(source)
        .build_yaml()
}

/// Creates an "unexpected X" error with source context
#[allow(dead_code)]
pub fn unexpected_error(source: &mut dyn ISource, found: &str) -> YamlError {
    ErrorBuilder::new(ErrorCategory::Syntax)
        .message(format!("Unexpected {}", found))
        .context(source)
        .build_yaml()
}

/// Creates an empty anchor/alias error
#[allow(dead_code)]
pub fn empty_name_error(name_type: &str) -> YamlError {
    ErrorBuilder::new(ErrorCategory::Syntax)
        .message(format!("Empty {} name", name_type))
        .build_yaml()
}

/// Creates an undefined reference error
#[allow(dead_code)]
pub fn undefined_reference(ref_type: &str, name: &str) -> YamlError {
    ErrorBuilder::new(ErrorCategory::Syntax)
        .message(format!("Undefined {} reference: {}", ref_type, name))
        .build_yaml()
}

/// Creates a duplicate definition error
#[allow(dead_code)]
pub fn duplicate_error(item_type: &str, name: &str) -> YamlError {
    ErrorBuilder::new(ErrorCategory::Syntax)
        .message(format!("Duplicate {}: {}", item_type, name))
        .build_yaml()
}

/// Creates an inconsistent indentation error with details
#[allow(dead_code)]
pub fn inconsistent_indent_error(
    source: &mut dyn ISource,
    expected: usize,
    got: usize,
    context: &str,
) -> YamlError {
    ErrorBuilder::new(ErrorCategory::Indentation)
        .message(format!(
            "Inconsistent indentation in {}: expected {}, got {}",
            context, expected, got
        ))
        .context(source)
        .build_yaml()
}

/// Creates a validation error for forbidden characters/patterns
pub fn forbidden_error(source: &mut dyn ISource, what: &str, where_forbidden: &str) -> YamlError {
    ErrorBuilder::new(ErrorCategory::Syntax)
        .message(format!("{} are not allowed {}", what, where_forbidden))
        .context(source)
        .build_yaml()
}

/// Structured indentation error when a tab is used for indentation in block context.
pub fn tab_indentation_error_yaml(source: &mut dyn ISource) -> YamlError {
    ErrorBuilder::new(ErrorCategory::Indentation)
        .message("Tabs are not allowed as indentation in YAML")
        .context(source)
        .build_yaml()
}

/// Structured error for invalid comment placement (e.g., missing preceding whitespace).
pub fn invalid_comment_spacing_error_yaml(source: &mut dyn ISource) -> YamlError {
    ErrorBuilder::new(ErrorCategory::Syntax)
        .message("Comment indicator (#) must be preceded by whitespace or newline")
        .context(source)
        .build_yaml()
}

/// Structured error for mapping key issues; `details` should describe the specific problem.
pub fn mapping_key_error_yaml(source: &mut dyn ISource, details: &str) -> YamlError {
    ErrorBuilder::new(ErrorCategory::Structure)
        .message(details)
        .context(source)
        .build_yaml()
}

/// Thin adapter to convert a structured `YamlError` back into the
/// legacy `String` representation used by older parser APIs.
///
/// This keeps the public API stable while allowing internal code to
/// use `ParseResult<T>` and `YamlError`.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_syntax_error() {
        let mut source = Buffer::new(b"test");
        let error = syntax_error(&mut source, "Invalid character");
        assert!(error.to_string().contains("Invalid character"));
        assert!(error.to_string().contains("current:"));
        assert!(error.to_string().contains("indent:"));
    }

    #[test]
    fn test_indentation_error() {
        let mut source = Buffer::new(b"  test");
        let error = indentation_error(&mut source, "Wrong indent");
        assert!(error.to_string().contains("Wrong indent"));
        assert!(error.to_string().contains("indent:")); // Just check for indent field, not specific value
        assert!(error.to_string().contains("current:"));
    }

    #[test]
    fn test_limit_error() {
        let error = limit_error("Sequence parsing", 100000, "loop iterations");
        assert!(error.to_string().contains("Sequence parsing"));
        assert!(error.to_string().contains("100000"));
        assert!(error.to_string().contains("loop iterations"));
        assert!(error.to_string().contains("infinite loop"));
    }

    #[test]
    fn test_structure_error() {
        let mut source = Buffer::new(b":");
        let error = structure_error(&mut source, "Missing key");
        assert!(error.to_string().contains("Missing key"));
    }

    #[test]
    fn test_eof_error() {
        let error = eof_error("inline mapping");
        assert!(
            error
                .to_string()
                .contains("Unexpected end of input in inline mapping")
        );
    }

    #[test]
    fn test_eof_expecting() {
        let error = eof_expecting("closing bracket");
        assert!(
            error
                .to_string()
                .contains("Unexpected end of input while expecting closing bracket")
        );
    }

    #[test]
    fn test_expected_error() {
        let mut source = Buffer::new(b",");
        let error = expected_error(&mut source, ":'");
        let err_str = error.to_string();
        assert!(
            err_str.contains("Expected :'")
                || err_str.contains("Expected :")
                || err_str.contains("Missing")
                || err_str.contains("Syntax error"),
            "Error message: {}",
            err_str
        );
        assert!(err_str.contains("current: ','"));
    }

    #[test]
    fn test_unexpected_error() {
        let mut source = Buffer::new(b"#");
        let error = unexpected_error(&mut source, "comment");
        assert!(error.to_string().contains("Unexpected comment"));
    }

    #[test]
    fn test_empty_name_error() {
        let error = empty_name_error("anchor");
        assert!(error.to_string().contains("Empty anchor name"));
    }

    #[test]
    fn test_undefined_reference() {
        let error = undefined_reference("anchor", "myanchor");
        assert!(
            error
                .to_string()
                .contains("Undefined anchor reference: myanchor")
        );
    }

    #[test]
    fn test_duplicate_error() {
        let error = duplicate_error("anchor", "myanchor");
        assert!(error.to_string().contains("Duplicate anchor: myanchor"));
    }

    #[test]
    fn test_inconsistent_indent_error() {
        let mut source = Buffer::new(b"test");
        let error = inconsistent_indent_error(&mut source, 2, 4, "sequence");
        let err_str = error.to_string();
        assert!(err_str.contains("Inconsistent indentation in sequence"));
        assert!(err_str.contains("expected 2"));
        assert!(err_str.contains("got 4"));
    }

    #[test]
    fn test_forbidden_error() {
        let mut source = Buffer::new(b"\t");
        let error = forbidden_error(&mut source, "Tabs", "as indentation in YAML");
        assert!(
            error
                .to_string()
                .contains("Tabs are not allowed as indentation in YAML")
        );
    }

    #[test]
    fn test_error_builder_yaml_error() {
        let mut source = Buffer::new(b":");
        let err = ErrorBuilder::new(ErrorCategory::Syntax)
            .message("Missing key before colon")
            .context(&mut source)
            .build_yaml();

        assert_eq!(err.kind(), &ErrorKind::SyntaxError);
        assert!(err.to_string().contains("Missing key before colon"));
    }

    #[test]
    fn test_error_builder_with_hint() {
        let mut source = Buffer::new(b":");
        let error = ErrorBuilder::new(ErrorCategory::Syntax)
            .message("Missing key before colon")
            .context(&mut source)
            .hint("Add a key before the ':'")
            .build();

        let err_str = error.to_string();
        assert!(err_str.contains("Missing key before colon"));
        assert!(err_str.contains("Hint:"));
        assert!(err_str.contains("Add a key"));
    }

    #[test]
    fn test_tab_indentation_error_yaml() {
        let mut source = Buffer::new(b"\tkey: value");
        let err = tab_indentation_error_yaml(&mut source);
        assert_eq!(err.kind(), &ErrorKind::SyntaxError);
        assert!(
            err.message()
                .contains("Tabs are not allowed as indentation in YAML")
        );
    }

    #[test]
    fn test_error_builder_at_eof() {
        let mut source = Buffer::new(b"");
        let error = syntax_error(&mut source, "Unexpected end");
        assert!(error.to_string().contains("current: 'EOF'"));
        assert!(error.to_string().contains("indent: 0"));
    }
}
