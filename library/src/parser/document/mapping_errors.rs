use crate::parser::token_stream::TokenStream;
use crate::error::enhanced::{EnhancedError, ErrorCode};
use crate::error::YamlError;

/// Centralized error: Mapping key without value (expected value after colon)
///
/// Returns an EnhancedError with code E001, preserving existing message text
/// and formatting used across the parser.
pub fn mapping_key_without_value_expected_value_after_colon(
    stream: &mut TokenStream,
) -> EnhancedError {
    EnhancedError::new(crate::parser::document::error_builder::mapping_key_error_yaml(
        stream.source_mut(),
        "YAML compliance error: Mapping key without value (expected value after colon)",
    ))
    .with_code(ErrorCode::E001)
}

/// Centralized error: Invalid indentation after comment when value is already complete
///
/// Returns an EnhancedError with code E007, matching existing behavior and text.
pub fn invalid_indentation_after_comment_in_mapping_value(
    stream: &mut TokenStream,
) -> EnhancedError {
    EnhancedError::new(crate::parser::document::error_builder::mapping_key_error_yaml(
        stream.source_mut(),
        "Invalid indentation after comment: indented content cannot extend a completed scalar mapping value",
    ))
    .with_code(ErrorCode::E007)
    .with_note("Check for misplaced comments or indentation.")
}

/// Centralized error: Invalid indentation extending a completed mapping value
///
/// This occurs when additional indentation appears after a key whose value is
/// already complete on the same line. YAML requires nested content to follow
/// a key with an omitted value; indented content cannot extend a completed
/// scalar value.
pub fn invalid_indentation_extending_completed_mapping_value(
    stream: &mut TokenStream,
) -> EnhancedError {
    EnhancedError::new(crate::parser::document::error_builder::mapping_key_error_yaml(
        stream.source_mut(),
        "Invalid indentation: indented content cannot extend a completed mapping value",
    ))
    .with_code(ErrorCode::E010)
    .with_note("Ensure nested content follows keys with omitted values (e.g., 'key:\n  nested: 1').")
}

/// Centralized error: Inconsistent dedent within nested mapping value for keys
///
/// Returns an EnhancedError with code E009, matching existing behavior and text.
pub fn inconsistent_dedent_within_mapping_value_for_keys(
    stream: &mut TokenStream,
) -> EnhancedError {
    EnhancedError::new(crate::parser::document::error_builder::mapping_key_error_yaml(
        stream.source_mut(),
        "Invalid indentation for nested mapping key: inconsistent dedent within mapping value",
    ))
    .with_code(ErrorCode::E009)
    .with_note("Ensure all keys under the nested mapping use the same indentation.")
}

/// Centralized error: Dedent below base indent within a mapping value
///
/// This indicates a key appearing at an indentation less than the parent mapping's
/// base indent, which is invalid for YAML block mappings.
pub fn inconsistent_dedent_below_base_indent_in_mapping(
    stream: &mut TokenStream,
) -> EnhancedError {
    EnhancedError::new(crate::parser::document::error_builder::mapping_key_error_yaml(
        stream.source_mut(),
        "Invalid indentation for mapping key: dedent below the base indent is not allowed",
    ))
    .with_code(ErrorCode::E009)
    .with_note("Keys in a block mapping must align at or above the parent mapping's indent.")
}

/// Centralized error: Invalid anchored alias key (anchors cannot be applied to alias nodes)
///
/// Returns an EnhancedError with code E004 and note, matching existing behavior and text.
pub fn invalid_anchored_alias_key_on_alias_nodes(stream: &mut TokenStream) -> EnhancedError {
    // Route through AnchorErrors to centralize the message text.
    // Behavior-neutral: underlying builder and final message remain identical.
    let base = crate::parser::document::anchor_errors::AnchorErrors::invalid_anchored_alias(stream);
    EnhancedError::new(base)
    .with_code(ErrorCode::E004)
    .with_note("Anchors are not allowed on alias nodes.")
}

/// Centralized error: Multiple anchors applied to a mapping key
///
/// Returns an EnhancedError with code E005 and note, matching existing behavior and text.
pub fn multiple_anchors_on_mapping_key(stream: &mut TokenStream) -> EnhancedError {
    EnhancedError::new(crate::parser::document::error_builder::mapping_key_error_yaml(
        stream.source_mut(),
        "A mapping key cannot have multiple anchors",
    ))
    .with_code(ErrorCode::E005)
    .with_note("A key can only have one anchor.")
}

/// Centralized error: Invalid content immediately after quoted scalar within mapping value
///
/// Behavior-neutral: preserves exact message text and error type.
pub fn invalid_trailing_plain_text_after_quoted_scalar(stream: &mut TokenStream) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        stream.source_mut(),
        "Invalid content immediately after quoted scalar: trailing plain text on the same line is not allowed",
    )
}

// Note: Nested ':' after a value on the same line is currently tolerated for
// compatibility with some suite cases and examples using permissive parsing.
/// Centralized error: Nested key separator ':' encountered immediately after a block mapping value on the same line
///
/// Rejects patterns like `a: b: c` or `a: 'b': c` in block context, which
/// incorrectly introduce a second key-value separator on the same line.
pub fn nested_key_separator_in_block_value_same_line(stream: &mut TokenStream) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        stream.source_mut(),
        "Invalid mapping value: unexpected ':' immediately after value on the same line",
    )
}

/// Centralized error: Expected '?' token for explicit key, got {cur}
pub fn expected_explicit_key_token(
    stream: &mut TokenStream,
    cur: Option<crate::parser::lexer::Token>,
) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        stream.source_mut(),
        &format!("Expected '?' token for explicit key, got {:?}", cur),
    )
}
