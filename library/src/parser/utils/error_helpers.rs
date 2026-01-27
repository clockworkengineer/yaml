/// Error helper for empty anchor name
pub fn empty_anchor_name() -> crate::error::YamlError {
    crate::error::YamlError::from(crate::error::messages::ERR_EMPTY_ANCHOR_NAME)
}

/// Error helper for undefined anchor
pub fn undefined_anchor(name: &str) -> crate::error::YamlError {
    crate::error::YamlError::from(format!("{}{}", crate::error::messages::ERR_UNDEFINED_ANCHOR_PREFIX, name))
}

/// Error helper for merge source not a mapping
pub fn merge_source_not_mapping(name: &str) -> crate::error::YamlError {
    crate::error::YamlError::from(format!("Merge source '{}' is not a mapping", name))
}

/// Error helper for invalid merge sequence item
pub fn invalid_merge_sequence_item() -> crate::error::YamlError {
    crate::error::YamlError::from("Invalid merge sequence item: expected alias or mapping")
}

/// Error helper for invalid merge value
pub fn invalid_merge_value(ty: &str) -> crate::error::YamlError {
    crate::error::YamlError::from(format!("Invalid merge value: expected alias, sequence or mapping, got {}", ty))
}
/// Centralized error helpers for the lexer
pub fn syntax_error<S: AsRef<str>>(
    source: &mut dyn crate::io::traits::ISource,
    msg: S,
) -> crate::error::YamlError {
    crate::parser::document::error_builder::syntax_error(source, msg.as_ref())
}

pub fn forbidden_error<S1: AsRef<str>, S2: AsRef<str>>(
    source: &mut dyn crate::io::traits::ISource,
    what: S1,
    context: S2,
) -> crate::error::YamlError {
    crate::parser::document::error_builder::forbidden_error(source, what.as_ref(), context.as_ref())
}
