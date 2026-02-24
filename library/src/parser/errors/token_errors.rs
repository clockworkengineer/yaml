
/*
 * Token Error Helpers
 *
 * Centralizes error construction for token-related parsing errors in YAML, providing helpers
 * for expected tokens, scalars, and other token stream expectations.
 *
 * Copyright (c) 2026 YAML Library Developers
 */

use crate::error::YamlError;
use crate::io::traits::ISource;
use crate::parser::lexer::Token;

/// Centralized helper: expected a specific token.
///
/// Wraps the existing message text used by TokenStream::expect to keep
/// behavior identical while reducing duplication in call sites.
pub fn expected_specific_token(source: &mut dyn ISource, expected: Token) -> YamlError {
    crate::parser::document::error_builder::expected_error(source, &format!("token {:?}", expected))
}

/// Centralized helper: expected a plain scalar.
pub fn expected_plain_scalar(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::expected_error(source, "plain scalar")
}

/// Centralized helper: expected a quoted scalar.
pub fn expected_quoted_scalar(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::expected_error(source, "quoted scalar")
}

/// Centralized helper: expected any scalar.
pub fn expected_scalar(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::expected_error(source, "scalar")
}

/// Centralized helper: parser did not advance (syntax context, typically at EOF).
///
/// Keeps the exact message text used historically while providing a single
/// place to construct this error.
pub fn parser_did_not_advance_syntax(source: &mut dyn ISource, context: &str) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        &format!(
            "Syntax error: Parser did not advance when parsing {} (possible malformed input)",
            context
        ),
    )
}

/// Centralized helper: parser did not advance (structure context).
///
/// Matches the existing message while centralizing construction.
pub fn parser_did_not_advance_structure(source: &mut dyn ISource, context: &str) -> YamlError {
    crate::parser::document::error_builder::structure_error(
        source,
        &format!(
            "Parser did not advance when parsing {} (possible malformed input)",
            context
        ),
    )
}

/// Centralized helper: expected plain scalar but found EOF.
pub fn expected_plain_scalar_eof(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::syntax_error(source, "Expected plain scalar, got EOF")
}

/// Centralized helper: expected quoted scalar but found EOF.
pub fn expected_quoted_scalar_eof(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::syntax_error(source, "Expected quoted scalar, got EOF")
}

/// Centralized helper: expected scalar but found EOF.
pub fn expected_scalar_eof(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::syntax_error(source, "Expected scalar, got EOF")
}

/// Centralized error: Empty token name (e.g., empty tag/anchor/alias)
///
/// Behavior-neutral: constructs the same syntax error using the provided
/// message text (e.g., "Empty tag name"). This keeps the exact output
/// unchanged while routing construction through this helper.
pub fn empty_token_name(source: &mut dyn ISource, message: &'static str) -> YamlError {
    crate::parser::document::error_builder::syntax_error(source, message)
}

/// Duplicate tag decorator encountered
pub fn duplicate_tag_found(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::syntax_error(source, "Duplicate tag found")
}

/// Duplicate anchor decorator encountered
pub fn duplicate_anchor_found(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::syntax_error(source, "Duplicate anchor found")
}

/// Invalid explicit tag handle usage (message provided by validator)
pub fn invalid_tag_handle_usage(source: &mut dyn ISource, message: &str) -> YamlError {
    crate::parser::document::error_builder::syntax_error(source, message)
}

/// Unterminated single-quoted string (unexpected EOF)
pub fn unterminated_single_quoted_eof(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        "YAML compliance error: Unterminated single-quoted string (unexpected EOF)",
    )
}

/// Unterminated double-quoted string (unexpected EOF after escape)
pub fn unterminated_double_quoted_eof_after_escape(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        "YAML compliance error: Unterminated double-quoted string (unexpected EOF after escape)",
    )
}

/// Unterminated double-quoted string (unexpected EOF)
pub fn unterminated_double_quoted_eof(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        "YAML compliance error: Unterminated double-quoted string (unexpected EOF)",
    )
}

/// Invalid \x escape (expected 2 hex digits)
pub fn invalid_escape_x_expected_2_hex(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        "YAML compliance error: Invalid \\x escape sequence, expected 2 hex digits",
    )
}

/// Invalid \u escape (expected 4 hex digits)
pub fn invalid_escape_u_expected_4_hex(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        "YAML compliance error: Invalid \\u escape sequence, expected 4 hex digits",
    )
}

/// Invalid \U escape (expected 8 hex digits)
pub fn invalid_escape_u_expected_8_hex(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        "YAML compliance error: Invalid \\U escape sequence, expected 8 hex digits",
    )
}

/// Invalid unicode codepoint U+XXXX (4-digit form)
pub fn invalid_unicode_codepoint_u4(source: &mut dyn ISource, code: u32) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        &format!(
            "YAML compliance error: Invalid unicode codepoint U+{:04X}",
            code
        ),
    )
}

/// Invalid unicode codepoint U+XXXXXXXX (8-digit form)
pub fn invalid_unicode_codepoint_u8(source: &mut dyn ISource, code: u32) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        &format!(
            "YAML compliance error: Invalid unicode codepoint U+{:08X}",
            code
        ),
    )
}

/// Invalid generic escape in double-quoted string
pub fn invalid_escape_generic(source: &mut dyn ISource, ch: char) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        &format!(
            "YAML compliance error: Invalid escape sequence '\\{}' in double-quoted string",
            ch
        ),
    )
}

/// Unexpected token in value
pub fn unexpected_token_in_value(source: &mut dyn ISource, token: &Token) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        &format!("Unexpected token in value: {:?}", token),
    )
}

/// Expected a scalar token, got {current}
pub fn expected_scalar_token(source: &mut dyn ISource, current: &str) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        &format!("Expected a scalar token, got {}", current),
    )
}

/// Document structure error: missing '---' between documents
pub fn document_unexpected_plain_after_top_level_sequence(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::structure_error(
        source,
        "Unexpected plain scalar after top-level sequence; missing '---' between documents",
    )
}

/// Unexpected comma after a tag in block context (invalid punctuation)
pub fn unexpected_comma_after_tag_in_block_value(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        "Unexpected comma after tag in block context",
    )
}
