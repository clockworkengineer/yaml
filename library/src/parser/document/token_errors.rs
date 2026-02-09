use crate::error::YamlError;
use crate::io::traits::ISource;
use crate::parser::lexer::Token;

/// Centralized helper: expected a specific token.
///
/// Wraps the existing message text used by TokenStream::expect to keep
/// behavior identical while reducing duplication in call sites.
pub fn expected_specific_token(source: &mut dyn ISource, expected: Token) -> YamlError {
    crate::parser::document::error_builder::expected_error(source, &format!("token {:?}", expected))
}

/// Centralized helper: expected a plain scalar.
pub fn expected_plain_scalar(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::expected_error(source, "plain scalar")
}

/// Centralized helper: expected a quoted scalar.
pub fn expected_quoted_scalar(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::expected_error(source, "quoted scalar")
}

/// Centralized helper: expected any scalar.
pub fn expected_scalar(source: &mut dyn ISource) -> YamlError {
    crate::parser::document::error_builder::expected_error(source, "scalar")
}

/// Centralized helper: parser did not advance (syntax context, typically at EOF).
///
/// Keeps the exact message text used historically while providing a single
/// place to construct this error.
pub fn parser_did_not_advance_syntax(source: &mut dyn ISource, context: &str) -> YamlError {
    crate::parser::document::error_builder::syntax_error(
        source,
        &format!(
            "Syntax error: Parser did not advance when parsing {} (possible malformed input)",
            context
        ),
    )
}

/// Centralized helper: parser did not advance (structure context).
///
/// Matches the existing message while centralizing construction.
pub fn parser_did_not_advance_structure(source: &mut dyn ISource, context: &str) -> YamlError {
    crate::parser::document::error_builder::structure_error(
        source,
        &format!(
            "Parser did not advance when parsing {} (possible malformed input)",
            context
        ),
    )
}
