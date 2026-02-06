use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;
use crate::parser::{ParseResult};

/// Flow contexts that share punctuation expectations
pub enum FlowContext {
    Sequence,
    Mapping,
}

/// Construct a behavior-identical syntax error for missing comma or end in flow collections.
///
/// Centralizes the exact error text used by existing call sites to keep messages and
/// semantics stable while making future policy tweaks easier.
pub fn expected_separator_or_end_error(
    stream: &mut TokenStream,
    ctx: FlowContext,
) -> crate::error::YamlError {
    match ctx {
        FlowContext::Sequence => {
            crate::parser::document::error_builder::syntax_error(
                stream.source_mut(),
                "Expected comma or ] in flow sequence",
            )
        }
        FlowContext::Mapping => {
            crate::parser::document::error_builder::syntax_error(
                stream.source_mut(),
                "Expected comma or } in flow mapping",
            )
        }
    }
}

/// Convenience helper: after completing an item/entry in a flow collection, ensure
/// the next token is a comma or closing bracket/brace. If not, return the centralized error.
///
/// This does not change behavior; it simply wraps the existing conditional + error construction.
pub fn ensure_separator_or_end(
    stream: &mut TokenStream,
    ctx: FlowContext,
    closing: Token,
) -> ParseResult<()> {
    match stream.current() {
        Some(Token::Comma) => Ok(()),
        Some(t) if t == &closing => Ok(()),
        _ => Err(expected_separator_or_end_error(stream, ctx)),
    }
}
