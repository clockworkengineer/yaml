
/*
 * Comment Error Helpers
 *
 * Centralizes error construction for YAML comment parsing, providing helpers
 * for consistent error messages and easier future maintenance.
 *
 * Copyright (c) 2026 YAML Library Developers
 */

use crate::error::YamlError;
use crate::io::traits::ISource;
use crate::parser::document::error_builder::syntax_error;

/// Centralized constructors for comment-related parsing errors.
/// Behavior-neutral: messages match existing strings exactly.
pub struct CommentErrors;

impl CommentErrors {
    /// A comment character '#' immediately after a closing quoted scalar
    /// without separating whitespace is invalid.
    pub fn comment_must_be_separated_from_quoted_scalar_by_whitespace(
        source: &mut dyn ISource,
    ) -> YamlError {
        syntax_error(
            source,
            "YAML syntax error: comment must be separated from quoted scalar by whitespace",
        )
    }

    /// A comment character '#' immediately after a flow closer ('}' or ']')
    /// without separating whitespace is invalid.
    pub fn comment_must_be_preceded_by_whitespace_after_flow_closer(
        source: &mut dyn ISource,
        closer: char,
    ) -> YamlError {
        syntax_error(
            source,
            &format!(
                "YAML syntax error: comment must be preceded by whitespace after '{}' in flow collection",
                closer
            ),
        )
    }
}
