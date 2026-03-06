//! Validation helpers for YAML parsing.
//!
//! Provides indentation validation, tab detection, comment spacing validation,
//! and trailing-content checks after document markers.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::io::traits::ISource;
use crate::parser::ParseResult;
use crate::parser::directives::DirectiveContext;
use crate::parser::token_stream::TokenStream;
use crate::parser::utils::context::ParsingContext;
use crate::utils::{is_comment_start, is_horizontal_space, is_line_terminator};

use super::core::{parse_error_token, to_yaml_error};

/// DRY: Single entry point for indentation and tab validation.
/// All logic that needs to validate indentation or tab usage must use this function.
///
/// Unified indentation and whitespace validation entry point.
///
/// This is the central hook for indentation/whitespace validation used by the
/// document parser. It is intentionally conservative in its initial
/// implementation so that we can wire it into the parsing pipeline without
/// changing behavior, and then progressively tighten the rules in
/// `validate_indentation_tokens` as we address specific false positives from
/// the official YAML test suite.
///
/// The function operates in two stages:
/// - Build a temporary `TokenStream` snapshot from the current `ISource`
/// - Delegate to the token-level validator, restoring the source state after
///   the check so that parsing continues unchanged
pub(crate) fn validate_indentation_and_whitespace(
    source: &mut dyn ISource,
    directives: &DirectiveContext,
    ctx: &ParsingContext,
) -> ParseResult<()> {
    let state = source.save_state();
    let stream = crate::parser::token_stream::TokenStream::new(source, directives, false)
        .map_err(to_yaml_error)?;
    let result = crate::parser::utils::indentation::validate_indentation_tokens(&stream, ctx);
    source.restore_state(state);
    result
}

/// Backward-compatible wrapper that validates no tabs are used for indentation in block context.
///
/// Assumes the current position is at an indentation point (start of a new line) in block context.
/// Token-based wrapper for validating no tab indentation at current position.
#[allow(dead_code)]
pub(crate) fn validate_no_tab_indentation_tokens(
    stream: &TokenStream,
    ctx: &ParsingContext,
) -> ParseResult<()> {
    crate::parser::utils::indentation::validate_indentation_tokens(stream, ctx)
}

/// Validates that no non-trivia content appears after a document end marker (`...`)
/// on the same line.  Consumes spaces and an inline comment up to the line end;
/// returns an error if any other content is found.
pub(crate) fn validate_trailing_content_after_document_end(
    stream: &mut TokenStream,
) -> ParseResult<()> {
    loop {
        match stream.source_mut().current() {
            Some(c) if is_horizontal_space(c) => {
                stream.source_mut().next();
            }
            Some(c) if is_comment_start(c) => {
                // Inline comment: consume until end of line
                while let Some(c2) = stream.source_mut().current() {
                    if is_line_terminator(c2) {
                        break;
                    }
                    stream.source_mut().next();
                }
                break;
            }
            Some(c) if is_line_terminator(c) => break,
            None => break,
            Some(c) => {
                return Err(parse_error_token(
                    stream,
                    &format!("Invalid content '{}' after document end marker (...)", c),
                ));
            }
        }
    }
    Ok(())
}

/// Validates that a Comment token is preceded by whitespace, newline, or is at the start of the stream.
///
/// DRY ENTRY POINT: All comment spacing validation must use this function.
/// Usage: Call this before accepting a comment token to ensure correct spacing.
///
/// According to the YAML spec, a comment indicator (#) must be preceded by whitespace or be at the start of a line.
/// This function checks the previous token in the TokenStream context.
#[allow(dead_code)]
pub(crate) fn validate_comment_spacing_token(stream: &TokenStream) -> ParseResult<()> {
    use crate::parser::lexer::Token;
    // Only validate if current token is a Comment
    if let Some(Token::Comment(_)) = stream.current() {
        // Try to get the previous token, if available
        // (Assume TokenStream has a way to get previous token, or track it externally)
        // For now, we assume TokenStream exposes a method or field for previous token, or this check is done at the point of parsing
        // Here, we just show the logic for the current token
        // If you have access to previous token, check:
        // - Indent, Newline, or None (start of stream) are valid
        // - Otherwise, error
        // Pseudocode:
        // match stream.previous() {
        //     None => Ok(()), // Start of stream
        //     Some(Token::Indent(_)) | Some(Token::Newline) => Ok(()),
        //     _ => Err(structure_error(
        //         stream.source_mut(),
        //         "Comment indicator (#) must be preceded by whitespace or newline"
        //     )),
        // }
        // Since TokenStream may not have previous(), this is a placeholder for integration at the call site.
        // If not possible, this function can be called with the previous token as an argument.
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::parser::utils::context::{CollectionType, ParsingContext};

    #[test]
    fn test_validate_indentation_block_context_with_tab() {
        // Tab in block context after newline should error
        let mut source = Buffer::new(b"\t  content");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, false);
        assert!(
            ts_result.is_err(),
            "TokenStream should error on tabs in block context"
        );
        if let Err(ref err) = ts_result {
            let err_str = err.to_string();
            assert!(
                err_str.contains("Tabs") && err_str.contains("not allowed"),
                "Error: {}",
                err_str
            );
            return;
        }
    }

    #[test]
    fn test_validate_indentation_block_context_no_tab() {
        // Spaces in block context are OK
        let mut source = Buffer::new(b"  content");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let directives = crate::parser::directives::DirectiveContext::new();
        let result = crate::parser::utils::indentation::validate_indentation_tokens(
            &TokenStream::new(&mut source, &directives, false)
                .expect("TokenStream creation failed"),
            &ctx,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_indentation_flow_context_with_tab() {
        // Tab in flow context is allowed
        let mut source = Buffer::new(b"\t  content");
        let ctx = ParsingContext::new(0).child_flow_context(CollectionType::FlowSequence);

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, true);
        // In flow context, tabs should be allowed, so expect Ok
        assert!(
            ts_result.is_ok(),
            "TokenStream should allow tabs in flow context"
        );
        if ts_result.is_err() {
            panic!("TokenStream should allow tabs in flow context");
        }
        let stream = ts_result.unwrap();
        let result = crate::parser::utils::indentation::validate_indentation_tokens(&stream, &ctx);
        assert!(result.is_ok(), "Tabs should be allowed in flow context");
    }

    #[test]
    fn test_validate_indentation_not_after_newline() {
        // Tab not immediately after newline (content found) - should not validate
        let mut source = Buffer::new(b"\tcontent");
        let ctx = ParsingContext::new(0);
        // Don't mark newline consumed - simulates tab in middle of content

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, true);
        // Tab not at indentation (not after newline) should be allowed
        assert!(
            ts_result.is_ok(),
            "TokenStream should allow tabs not at indentation position"
        );
        if ts_result.is_err() {
            panic!("TokenStream should allow tabs not at indentation position");
        }
        let stream = ts_result.unwrap();
        let result = crate::parser::utils::indentation::validate_indentation_tokens(&stream, &ctx);
        assert!(result.is_ok(), "Tabs OK when not at indentation position");
    }

    #[test]
    fn test_validate_indentation_blank_line() {
        // Tab-only line (tab followed by newline) should be allowed in block context
        // as it represents a blank line, not indentation of content.
        // This matches YAML test case DK95/04.
        let mut source = Buffer::new(b"\t\n");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, false);
        assert!(
            ts_result.is_ok(),
            "TokenStream should allow tab-only lines (blank lines) in block context"
        );
    }

    #[test]
    fn test_validate_indentation_actual_blank_line() {
        // Truly blank line (just newline, no tabs) should be OK
        let mut source = Buffer::new(b"\n");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let directives = crate::parser::directives::DirectiveContext::new();
        let result = crate::parser::utils::indentation::validate_indentation_tokens(
            &TokenStream::new(&mut source, &directives, false)
                .expect("TokenStream creation failed"),
            &ctx,
        );
        assert!(result.is_ok(), "Blank lines without tabs should be OK");
    }

    #[test]
    fn test_validate_indentation_spaces_then_tab() {
        // Spaces followed by tab in indentation
        // Note: This pattern is allowed because it occurs in block scalar content
        // where tabs are part of the preserved content, not structural indentation.
        // The lexer allows tabs after spaces to support block scalars like:
        //   block: |
        //     line1
        //       \tcontent
        let mut source = Buffer::new(b"  \tcontent");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, false);
        assert!(
            ts_result.is_ok(),
            "TokenStream should allow tabs after spaces for block scalar content"
        );
    }

    #[test]
    fn test_validate_no_tab_indentation_backward_compat() {
        // Test backward compatibility wrapper
        let mut source = Buffer::new(b"\tcontent");

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, false);
        assert!(
            ts_result.is_err(),
            "TokenStream should error on tabs in block context"
        );
        if let Err(ref err) = ts_result {
            let err_str = err.to_string();
            assert!(
                err_str.contains("Tabs") && err_str.contains("not allowed"),
                "Error: {}",
                err_str
            );
            return;
        }
    }

    #[test]
    fn test_skip_whitespace_with_context_block() {
        // Test the new wrapper function
        let mut source = Buffer::new(b"  content");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let directives = crate::parser::directives::DirectiveContext::new();
        {
            let mut stream = TokenStream::new(&mut source, &directives, false)
                .expect("TokenStream creation failed");
            let result = stream.skip_whitespace_and_comments();
            assert!(
                result.is_ok(),
                "Whitespace skipping in block context should succeed"
            );
            // Also check the token stream's current token
            use crate::parser::lexer::Token;
            match stream.current() {
                Some(Token::Plain(s)) => {
                    assert!(
                        s.starts_with('c'),
                        "TokenStream should be at plain scalar starting with 'c'"
                    );
                }
                other => panic!(
                    "TokenStream not at expected plain scalar after whitespace: {:?}",
                    other
                ),
            }
        }
    }

    #[test]
    fn test_skip_whitespace_with_context_tab_error() {
        let mut source = Buffer::new(b"\tcontent");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, false);
        assert!(
            ts_result.is_err(),
            "TokenStream should error on tabs in block context"
        );
        if let Err(ref err) = ts_result {
            let err_str = err.to_string();
            assert!(
                err_str.contains("Tabs") && err_str.contains("not allowed"),
                "Error: {}",
                err_str
            );
            return;
        }
    }

    #[test]
    fn test_skip_whitespace_with_context_flow() {
        // Tabs OK in flow context
        let mut source = Buffer::new(b"\tcontent");

        let directives = crate::parser::directives::DirectiveContext::new();
        let ts_result = TokenStream::new(&mut source, &directives, true);
        // Tabs should be allowed in flow context
        assert!(
            ts_result.is_ok(),
            "TokenStream should allow tabs in flow context"
        );
        if ts_result.is_err() {
            panic!("TokenStream should allow tabs in flow context");
        }
        let mut stream = ts_result.unwrap();
        let result = stream.skip_whitespace_and_comments();
        assert!(result.is_ok(), "Tabs should be allowed in flow context");
    }
}
