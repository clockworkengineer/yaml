use crate::parser::token_stream::TokenStream;
use crate::error::enhanced::{EnhancedError, ErrorCode};

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
