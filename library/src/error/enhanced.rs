//! Enhanced error handling with suggestions, recovery, and rich context
//!
//! This module provides advanced error handling features including:
//! - Error suggestions ("did you mean...?")
//! - Error recovery strategies
//! - Source code snippets in error messages
//! - Error codes for programmatic handling
//! - Structured error reporting

#[cfg(feature = "std")]
use std::fmt;

#[cfg(not(feature = "std"))]
use core::fmt;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::error::{ErrorKind, YamlError};

/// Error code for programmatic error handling
///
/// Allows applications to handle specific errors without string matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// E001: Missing colon in mapping
    E001,
    /// E002: Unterminated quoted string
    E002,
    /// E003: Invalid escape sequence
    E003,
    /// E004: Undefined alias reference
    E004,
    /// E005: Duplicate anchor name
    E005,
    /// E006: Invalid tag syntax
    E006,
    /// E007: Unexpected indentation
    E007,
    /// E008: Invalid character in key
    E008,
    /// E009: Unclosed flow collection
    E009,
    /// E010: Invalid document marker
    E010,
    /// E011: Circular reference detected
    E011,
    /// E012: Exceeded nesting limit
    E012,
    /// E013: Invalid boolean value
    E013,
    /// E014: Invalid numeric value
    E014,
    /// E015: Invalid null value
    E015,
}

impl ErrorCode {
    /// Get the error code as a string (e.g., "E001")
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::E001 => "E001",
            ErrorCode::E002 => "E002",
            ErrorCode::E003 => "E003",
            ErrorCode::E004 => "E004",
            ErrorCode::E005 => "E005",
            ErrorCode::E006 => "E006",
            ErrorCode::E007 => "E007",
            ErrorCode::E008 => "E008",
            ErrorCode::E009 => "E009",
            ErrorCode::E010 => "E010",
            ErrorCode::E011 => "E011",
            ErrorCode::E012 => "E012",
            ErrorCode::E013 => "E013",
            ErrorCode::E014 => "E014",
            ErrorCode::E015 => "E015",
        }
    }

    /// Get a description of what this error means
    pub fn description(&self) -> &'static str {
        match self {
            ErrorCode::E001 => "Missing colon separator in mapping",
            ErrorCode::E002 => "Quoted string is not properly terminated",
            ErrorCode::E003 => "Escape sequence is invalid or malformed",
            ErrorCode::E004 => "Alias references an undefined anchor",
            ErrorCode::E005 => "Anchor name is used more than once",
            ErrorCode::E006 => "Tag syntax is invalid or unsupported",
            ErrorCode::E007 => "Indentation is inconsistent or unexpected",
            ErrorCode::E008 => "Key contains invalid characters",
            ErrorCode::E009 => "Flow collection ([], {}) is not closed",
            ErrorCode::E010 => "Document marker (---/...) is malformed",
            ErrorCode::E011 => "Circular reference in aliases detected",
            ErrorCode::E012 => "Nesting depth exceeds configured limit",
            ErrorCode::E013 => "Boolean value must be true/false/yes/no/on/off",
            ErrorCode::E014 => "Numeric value format is invalid",
            ErrorCode::E015 => "Null value must be null/~ or empty",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Span representing a range in the source document
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Starting line (1-based)
    pub start_line: usize,
    /// Starting column (1-based)
    pub start_col: usize,
    /// Ending line (1-based)
    pub end_line: usize,
    /// Ending column (1-based)
    pub end_col: usize,
}

impl Span {
    /// Create a new span
    pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    /// Create a span for a single point
    pub fn point(line: usize, col: usize) -> Self {
        Self {
            start_line: line,
            start_col: col,
            end_line: line,
            end_col: col,
        }
    }

    /// Check if this span is a single point
    pub fn is_point(&self) -> bool {
        self.start_line == self.end_line && self.start_col == self.end_col
    }
}

/// Suggestion for fixing an error
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorSuggestion {
    /// Description of the suggestion
    pub message: String,
    /// Suggested replacement text (if applicable)
    pub replacement: Option<String>,
    /// Span where the fix should be applied
    pub span: Option<Span>,
}

impl ErrorSuggestion {
    /// Create a new suggestion
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            replacement: None,
            span: None,
        }
    }

    /// Add a replacement suggestion
    pub fn with_replacement(mut self, replacement: impl Into<String>) -> Self {
        self.replacement = Some(replacement.into());
        self
    }

    /// Add a span for the suggestion
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}

/// Enhanced error with rich context and suggestions
#[derive(Debug, Clone)]
pub struct EnhancedError {
    /// The base error
    base: YamlError,
    /// Error code for programmatic handling
    code: Option<ErrorCode>,
    /// Source code snippet around the error
    snippet: Option<String>,
    /// Span of the error in source
    span: Option<Span>,
    /// Suggestions for fixing the error
    suggestions: Vec<ErrorSuggestion>,
    /// Related notes or context
    notes: Vec<String>,
}

impl EnhancedError {
    /// Create a new enhanced error from a base error
    pub fn new(base: YamlError) -> Self {
        Self {
            base,
            code: None,
            snippet: None,
            span: None,
            suggestions: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Add an error code
    pub fn with_code(mut self, code: ErrorCode) -> Self {
        self.code = Some(code);
        self
    }

    /// Add a source code snippet
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    /// Add a span
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: ErrorSuggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Add a note
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Get the base error
    pub fn base(&self) -> &YamlError {
        &self.base
    }

    /// Get the error code
    pub fn code(&self) -> Option<ErrorCode> {
        self.code
    }

    /// Get suggestions
    pub fn suggestions(&self) -> &[ErrorSuggestion] {
        &self.suggestions
    }

    /// Get notes
    pub fn notes(&self) -> &[String] {
        &self.notes
    }

    /// Format the error with all context
    pub fn format_detailed(&self) -> String {
        let mut output = String::new();

        // Error header with code
        if let Some(code) = self.code {
            output.push_str(&format!("[{}] ", code));
        }
        output.push_str(&self.base.to_string());
        output.push('\n');

        // Code snippet
        if let Some(snippet) = &self.snippet {
            output.push_str("\n");
            output.push_str(snippet);
            output.push_str("\n");
        }

        // Suggestions
        if !self.suggestions.is_empty() {
            output.push_str("\nSuggestions:\n");
            for (i, suggestion) in self.suggestions.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, suggestion.message));
                if let Some(replacement) = &suggestion.replacement {
                    output.push_str(&format!("     Try: {}\n", replacement));
                }
            }
        }

        // Notes
        if !self.notes.is_empty() {
            output.push_str("\nNote:\n");
            for note in &self.notes {
                output.push_str(&format!("  {}\n", note));
            }
        }

        output
    }
}

impl fmt::Display for EnhancedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_detailed())
    }
}

/// Error recovery strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Skip the current line and continue
    SkipLine,
    /// Skip to the next key-value pair
    SkipToNextMapping,
    /// Skip to the end of current collection
    SkipCollection,
    /// Insert a default value and continue
    InsertDefault,
    /// Stop parsing (no recovery)
    Abort,
}

/// Error recovery context
#[derive(Debug)]
pub struct ErrorRecovery {
    /// Strategy to use for recovery
    pub strategy: RecoveryStrategy,
    /// Whether recovery was successful
    pub recovered: bool,
    /// Message about the recovery action taken
    pub message: String,
}

impl ErrorRecovery {
    /// Create a new error recovery
    pub fn new(strategy: RecoveryStrategy) -> Self {
        Self {
            strategy,
            recovered: false,
            message: String::new(),
        }
    }

    /// Mark recovery as successful
    pub fn success(mut self, message: impl Into<String>) -> Self {
        self.recovered = true;
        self.message = message.into();
        self
    }

    /// Mark recovery as failed
    pub fn failed(mut self, message: impl Into<String>) -> Self {
        self.recovered = false;
        self.message = message.into();
        self
    }
}

/// Error suggestion builder for common patterns
pub struct SuggestionBuilder;

impl SuggestionBuilder {
    /// Suggest adding missing colon in mapping
    pub fn missing_colon(key: &str) -> ErrorSuggestion {
        ErrorSuggestion::new(format!("Add ':' after the key '{}'", key))
            .with_replacement(format!("{}: ", key))
    }

    /// Suggest closing quote
    pub fn unclosed_quote(quote_char: char) -> ErrorSuggestion {
        ErrorSuggestion::new(format!("Add closing {} to terminate the string", quote_char))
    }

    /// Suggest fixing typo in boolean value
    pub fn boolean_typo(actual: &str) -> ErrorSuggestion {
        let suggestion = match actual.to_lowercase().as_str() {
            "ture" | "tru" => "true",
            "flase" | "fals" => "false",
            "ye" | "ys" => "yes",
            "n" => "no",
            _ => return ErrorSuggestion::new("Use one of: true, false, yes, no, on, off"),
        };
        
        ErrorSuggestion::new(format!("Did you mean '{}'?", suggestion))
            .with_replacement(suggestion.to_string())
    }

    /// Suggest fixing typo in null value
    pub fn null_typo(actual: &str) -> ErrorSuggestion {
        let suggestion = match actual.to_lowercase().as_str() {
            "nul" | "nil" | "none" => "null",
            _ => return ErrorSuggestion::new("Use 'null' or '~' for null values"),
        };
        
        ErrorSuggestion::new(format!("Did you mean '{}'?", suggestion))
            .with_replacement(suggestion.to_string())
    }

    /// Suggest fixing anchor/alias
    pub fn undefined_alias(alias: &str, available: &[String]) -> ErrorSuggestion {
        if available.is_empty() {
            return ErrorSuggestion::new("No anchors are defined in this document")
                .with_replacement(format!("&{} ... *{}", alias, alias));
        }

        // Find closest match using simple string distance
        let closest = available.iter()
            .min_by_key(|anchor| edit_distance(alias, anchor))
            .unwrap();

        if edit_distance(alias, closest) <= 3 {
            ErrorSuggestion::new(format!("Did you mean '*{}'?", closest))
                .with_replacement(format!("*{}", closest))
        } else {
            ErrorSuggestion::new(format!("Available anchors: {}", available.join(", ")))
        }
    }

    /// Suggest fixing indentation
    pub fn indentation(expected: usize, actual: usize) -> ErrorSuggestion {
        let action = if actual < expected {
            format!("Increase indentation by {} spaces", expected - actual)
        } else {
            format!("Decrease indentation by {} spaces", actual - expected)
        };
        
        ErrorSuggestion::new(format!("Expected {} spaces, found {}. {}", expected, actual, action))
    }
}

/// Simple edit distance calculation (Levenshtein distance)
fn edit_distance(a: &str, b: &str) -> usize {
    let len_a = a.len();
    let len_b = b.len();
    
    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

    for i in 0..=len_a {
        matrix[i][0] = i;
    }
    for j in 0..=len_b {
        matrix[0][j] = j;
    }

    for (i, ca) in a.chars().enumerate() {
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            matrix[i + 1][j + 1] = core::cmp::min(
                core::cmp::min(
                    matrix[i][j + 1] + 1,     // deletion
                    matrix[i + 1][j] + 1,     // insertion
                ),
                matrix[i][j] + cost,          // substitution
            );
        }
    }

    matrix[len_a][len_b]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code() {
        assert_eq!(ErrorCode::E001.as_str(), "E001");
        assert_eq!(ErrorCode::E001.to_string(), "E001");
        assert!(ErrorCode::E001.description().contains("colon"));
    }

    #[test]
    fn test_span() {
        let span = Span::new(1, 5, 1, 10);
        assert_eq!(span.start_line, 1);
        assert_eq!(span.end_col, 10);
        assert!(!span.is_point());

        let point = Span::point(5, 10);
        assert!(point.is_point());
    }

    #[test]
    fn test_error_suggestion() {
        let suggestion = ErrorSuggestion::new("Add colon")
            .with_replacement("key: value");
        
        assert_eq!(suggestion.message, "Add colon");
        assert_eq!(suggestion.replacement, Some("key: value".to_string()));
    }

    #[test]
    fn test_enhanced_error() {
        let base = YamlError::new(ErrorKind::SyntaxError, "test error")
            .with_position(5, 10);
        
        let enhanced = EnhancedError::new(base)
            .with_code(ErrorCode::E001)
            .with_suggestion(ErrorSuggestion::new("Try this"))
            .with_note("Additional context");

        assert_eq!(enhanced.code(), Some(ErrorCode::E001));
        assert_eq!(enhanced.suggestions().len(), 1);
        assert_eq!(enhanced.notes().len(), 1);
    }

    #[test]
    fn test_suggestion_builder_boolean() {
        let suggestion = SuggestionBuilder::boolean_typo("ture");
        assert!(suggestion.message.contains("true"));
        assert_eq!(suggestion.replacement, Some("true".to_string()));

        let suggestion = SuggestionBuilder::boolean_typo("flase");
        assert!(suggestion.message.contains("false"));
    }

    #[test]
    fn test_suggestion_builder_null() {
        let suggestion = SuggestionBuilder::null_typo("nil");
        assert!(suggestion.message.contains("null"));
        assert_eq!(suggestion.replacement, Some("null".to_string()));
    }

    #[test]
    fn test_suggestion_builder_alias() {
        let available = vec!["anchor1".to_string(), "myanchor".to_string()];
        let suggestion = SuggestionBuilder::undefined_alias("ancor1", &available);
        assert!(suggestion.message.contains("anchor1"));
    }

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("hello", "hello"), 0);
        assert_eq!(edit_distance("", "test"), 4);
        assert_eq!(edit_distance("test", ""), 4);
    }

    #[test]
    fn test_recovery_strategy() {
        let recovery = ErrorRecovery::new(RecoveryStrategy::SkipLine)
            .success("Skipped invalid line");
        
        assert!(recovery.recovered);
        assert_eq!(recovery.strategy, RecoveryStrategy::SkipLine);
    }

    #[test]
    fn test_enhanced_error_format() {
        let base = YamlError::new(ErrorKind::SyntaxError, "Missing colon");
        let enhanced = EnhancedError::new(base)
            .with_code(ErrorCode::E001)
            .with_suggestion(ErrorSuggestion::new("Add ':' after key"))
            .with_note("Mappings require key: value pairs");

        let formatted = enhanced.format_detailed();
        assert!(formatted.contains("[E001]"));
        assert!(formatted.contains("Suggestions:"));
        assert!(formatted.contains("Note:"));
    }
}
