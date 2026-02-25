
//! Error Helper Functions
//!
//! Provides helper functions for constructing common YAML parsing errors, including
//! empty anchor names, undefined anchors, and merge source validation.
//!
//! Copyright (c) 2026 YAML Library Developers

/// Error helper for empty anchor name
#[allow(dead_code)]
pub fn empty_anchor_name() -> crate::error::YamlError {
    crate::error::YamlError::from(crate::error::messages::ERR_EMPTY_ANCHOR_NAME)
}

/// Error helper for undefined anchor
#[allow(dead_code)]
pub fn undefined_anchor(name: &str) -> crate::error::YamlError {
    crate::error::YamlError::from(format!(
        "{}{}",
        crate::error::messages::ERR_UNDEFINED_ANCHOR_PREFIX,
        name
    ))
}

/// Error helper for merge source not a mapping
#[allow(dead_code)]
pub fn merge_source_not_mapping(name: &str) -> crate::error::YamlError {
    crate::error::YamlError::from(format!("Merge source '{}' is not a mapping", name))
}

/// Error helper for invalid merge sequence item
#[allow(dead_code)]
pub fn invalid_merge_sequence_item() -> crate::error::YamlError {
    crate::error::YamlError::from("Invalid merge sequence item: expected alias or mapping")
}

/// Error helper for invalid merge value
#[allow(dead_code)]
pub fn invalid_merge_value(ty: &str) -> crate::error::YamlError {
    crate::error::YamlError::from(format!(
        "Invalid merge value: expected alias, sequence or mapping, got {}",
        ty
    ))
}
/// Centralized error helpers for the lexer
pub fn syntax_error<S: AsRef<str>>(
    source: &mut dyn crate::io::traits::ISource,
    msg: S,
) -> crate::error::YamlError {
    crate::parser::utils::error_builder::syntax_error(source, msg.as_ref())
}
