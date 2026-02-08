use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;
use crate::parser::{ParseResult};
use crate::io::traits::ISource;
use crate::error::YamlError;

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

/// Centralized error: Unexpected extra closing bracket ']' in a flow sequence.
pub fn unexpected_extra_closing_bracket_in_flow_sequence(
    stream: &mut TokenStream,
) -> crate::error::YamlError {
    crate::parser::document::error_builder::syntax_error(
        stream.source_mut(),
        "Unexpected extra closing bracket ']' in flow sequence",
    )
}

/// Centralized error: Leading or double comma in flow sequence is not allowed.
pub fn leading_or_double_comma_in_flow_sequence(
    stream: &mut TokenStream,
) -> crate::error::YamlError {
    crate::parser::document::error_builder::syntax_error(
        stream.source_mut(),
        "Leading or double comma in flow sequence is not allowed",
    )
}

/// Centralized error: Unexpected EOF in flow mapping (unclosed '{').
pub fn unexpected_eof_in_flow_mapping_unclosed(
    stream: &mut TokenStream,
) -> crate::error::YamlError {
    crate::parser::document::error_builder::syntax_error(
        stream.source_mut(),
        "Syntax error: Unexpected end of input in flow mapping (unclosed '{')",
    )
}

/// Centralized error: Unexpected EOF in flow sequence.
///
/// Mirrors existing behavior and message text used in callers
/// to keep error output identical while centralizing construction.
pub fn unexpected_eof_in_flow_sequence(
    _stream: &mut TokenStream,
) -> crate::error::YamlError {
    // Note: For flow sequence EOF, existing code uses the generic EOF helper
    // without source context. Preserve exact message for neutrality.
    crate::parser::document::error_builder::eof_error("flow sequence")
}

/// Centralized error: Invalid bare '-' entries inside a flow sequence.
///
/// Some YAML suite cases expect that a flow sequence containing only bare
/// dash scalars (e.g., `[-, -]`) is rejected rather than treated as valid
/// scalar entries. This keeps the exact existing message while centralizing
/// construction.
pub fn invalid_bare_dash_entries_in_flow_sequence(
    stream: &mut TokenStream,
) -> crate::error::YamlError {
    crate::parser::document::error_builder::mapping_key_error_yaml(
        stream.source_mut(),
        "Invalid use of '-' indicators inside flow sequence",
    )
}

/// Centralized error: Invalid immediate content following a flow closer ('}' or ']').
///
/// When a non-whitespace character directly follows a flow closer, certain characters
/// are allowed (newline, ',', ']', '}', '#', ':'). Otherwise, emit a syntax error
/// indicating that whitespace or a newline is required.
pub fn invalid_content_immediately_after_flow_closer(
    source: &mut dyn ISource,
    closer: char,
    found: char,
) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        &format!(
            "YAML syntax error: Invalid content '{}' immediately after '{}' - whitespace or newline required",
            found, closer
        ),
    )
}
