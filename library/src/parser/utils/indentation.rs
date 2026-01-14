//! Module: parser/utils/indentation.rs
//!
//! Shared token-based indentation and whitespace validation utilities
//! used by the document parser. This is the canonical home for
//! indentation rules; document helpers should delegate here.

use crate::parser::document::context::ParsingContext;
use crate::parser::token_stream::TokenStream;

/// Token-based indentation validation using `TokenStream`.
///
/// At the moment this function is intentionally minimal and only set up as a
/// customization point; it will be extended to enforce stricter
/// indentation/whitespace rules (using `ParsingContext`) as we work through
/// the official YAML test suite false positives.
pub(crate) fn validate_indentation_tokens(
    stream: &TokenStream,
    ctx: &ParsingContext,
) -> crate::parser::ParseResult<()> {
    use crate::parser::lexer::Token;

    if ctx.in_flow {
        return Ok(());
    }
    if !ctx.should_validate_tab_indentation() {
        return Ok(());
    }
    if let Some(Token::Indent(_)) = stream.current() {
        // In YAML, indentation is always spaces. Tabs are forbidden.
        // If the lexer encodes tab usage in Indent tokens, we can extend this
        // function to report an error when tabs are used for indentation.
        // Until then, this function is effectively a no-op and serves as a
        // centralized place to evolve indentation rules.
        Ok(())
    } else {
        Ok(())
    }
}

/// Convenience wrapper for validating indentation at the beginning of a line.
///
/// This is the canonical entry point for callers that already operate on
/// a `TokenStream` positioned at a logical line start. It currently
/// delegates directly to `validate_indentation_tokens`, but exists as a
/// stable, self-documenting API for future indentation rule tweaks.
pub(crate) fn validate_indentation_at_line_start(
    stream: &TokenStream,
    ctx: &ParsingContext,
) -> crate::parser::ParseResult<()> {
    validate_indentation_tokens(stream, ctx)
}
