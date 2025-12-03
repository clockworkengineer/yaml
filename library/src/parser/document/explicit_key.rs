//! Helper functions for parsing explicit mapping keys (? indicator)

use crate::parser::token_stream::TokenStream;
use crate::parser::lexer::Token;
use crate::nodes::node::Node;
use crate::parser::document::error_builder::syntax_error;

/// Checks if the current token starts an explicit key (Token::QuestionMark)
#[allow(dead_code)]
pub(crate) fn is_explicit_key_start(stream: &mut TokenStream) -> bool {
    matches!(stream.current(), Some(Token::QuestionMark))
}

/// Parses an explicit mapping key-value pair using tokens
/// Returns (key_node, value_node)
#[allow(dead_code)]
pub(crate) fn parse_explicit_mapping_entry(
    stream: &mut TokenStream,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<(Node, Node), String> {
    // Check for explicit key indicator
    if !is_explicit_key_start(stream) {
        return Err("Expected '?' token for explicit key".to_string());
    }
    stream.next()?;
    stream.skip_whitespace()?;

    // Parse the key (may be empty)
    let key_node = match stream.current() {
        Some(Token::Newline) => {
            stream.next()?;
            stream.skip_whitespace()?;
            // Parse document contents as key (empty explicit key)
            crate::parser::document::tokens::value::parse_value_with_tokens(stream, directives)?
        }
        _ => {
            crate::parser::document::tokens::value::parse_value_with_tokens(stream, directives)?
        }
    };

    // Skip whitespace/comments after key
    stream.skip_whitespace()?;

    // Look for the colon indicator
    let value_node = match stream.current() {
        Some(Token::Colon) => {
            stream.next()?;
            stream.skip_whitespace()?;
            match stream.current() {
                Some(Token::Newline) => {
                    stream.next()?;
                    stream.skip_whitespace()?;
                    crate::parser::document::tokens::value::parse_value_with_tokens(stream, directives)?
                }
                _ => {
                    crate::parser::document::tokens::value::parse_value_with_tokens(stream, directives)?
                }
            }
        }
        // No value indicator, key only
        _ => Node::None,
    };

    Ok((key_node, value_node))
}
