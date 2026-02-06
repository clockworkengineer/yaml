use crate::error::YamlError;
use crate::io::traits::ISource;
use crate::parser::document::error_builder::{indentation_error, syntax_error};

/// Centralized constructors for block scalar parsing errors.
/// Behavior-neutral: messages match existing strings exactly.
pub struct BlockScalarErrors;

impl BlockScalarErrors {
    /// Unexpected trailing text immediately after '|' or '>' in block scalar header.
    pub fn invalid_header_unexpected_text(source: &mut dyn ISource) -> YamlError {
        syntax_error(
            source,
            "Invalid block scalar header: unexpected text immediately after '|' or '>'",
        )
    }

    /// Invalid explicit indent indicator in block scalar header.
    pub fn invalid_indent_indicator(source: &mut dyn ISource) -> YamlError {
        syntax_error(
            source,
            "Invalid block scalar indentation indicator: must be a single digit from 1-9",
        )
    }

    /// Literal block scalar: blank lines before content indented more than the content.
    pub fn invalid_literal_blank_indent(
        source: &mut dyn ISource,
        blank_max: usize,
        first_content_indent: usize,
    ) -> YamlError {
        let msg = alloc::format!(
            "Invalid indentation in literal block scalar: blank lines before content are more indented than the content (blank max: {}, first content indent: {})",
            blank_max, first_content_indent
        );
        indentation_error(source, &msg)
    }
}
