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
}
