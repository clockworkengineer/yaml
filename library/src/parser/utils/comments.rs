//! Comment Handling Helpers
//!
//! Provides shared helpers for comment handling and comment+indentation validation
//! at the YAML document level. Ensures proper comment placement and indentation.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::error::YamlError;
use crate::io::traits::ISource;
use crate::parser::ParseResult;
use crate::parser::directives::DirectiveContext;
use crate::parser::token_stream::TokenStream;

/// Validate that a top-level comment line is not followed by
/// improperly indented value-like content (e.g., 8XDJ-style
/// false positives where an indented scalar after a comment
/// would otherwise be treated as a separate top-level node).
///
/// This helper is intended to be called when the current
/// character in the underlying source is `'#'` and the
/// caller is at a document-level context with the provided
/// `indent_level`.
pub(crate) fn validate_top_level_comment_followed_by_indented_content(
    source: &mut dyn ISource,
    directives: &DirectiveContext,
    indent_level: usize,
) -> ParseResult<()> {
    let mut stream = TokenStream::new(source, directives, false)?;

    // Consume consecutive comment tokens starting at this position
    while matches!(
        stream.current(),
        Some(crate::parser::lexer::Token::Comment(_))
    ) {
        stream.next()?;
    }

    // If the comment is followed by a newline, inspect the indentation
    // of the next line to catch patterns like 8XDJ where an indented
    // scalar appears after a top-level comment with no enclosing
    // block structure.
    if let Some(crate::parser::lexer::Token::Newline) = stream.current() {
        // Move to the first token on the following line
        stream.next()?;
        match stream.current().cloned() {
            Some(crate::parser::lexer::Token::Indent(level)) if level > indent_level => {
                // Consume the Indent token, then skip any additional
                // newlines or comments to see what real content follows.
                stream.next()?;
                stream.skip_newlines_and_comments()?;
                match stream.current() {
                    // If the next significant token is a value-like token
                    // (plain scalar or the start of a flow collection),
                    // report this as invalid indented content after a
                    // top-level comment.
                    Some(crate::parser::lexer::Token::Plain(_))
                    | Some(crate::parser::lexer::Token::SingleQuoted(_))
                    | Some(crate::parser::lexer::Token::DoubleQuoted(_))
                    | Some(crate::parser::lexer::Token::FlowMappingStart)
                    | Some(crate::parser::lexer::Token::FlowSequenceStart) => {
                        let msg = crate::parser::utils::helpers::parse_error_token(
                            &stream,
                            "Unexpected indented content after top-level comment.",
                        );
                        return Err(YamlError::from(msg));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // Preserve existing behavior of skipping over the comment and
    // any associated trivia before returning to the caller.
    stream.skip_trivia()?;
    Ok(())
}
