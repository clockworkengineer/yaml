/// Normalizes a Node to a double-quoted Node::Str for use as a mapping key.
fn normalize_node_to_str(node: &Node) -> Node {
    use crate::nodes::node::BlockStyle;
    use crate::parser::document::helpers::node_to_inline_string;
    match node {
        Node::Array(_) | Node::Mapping(_) => {
            let inline = node_to_inline_string(node);
            Node::Str(
                inline,
                crate::nodes::node::QuoteType::Double,
                BlockStyle::None,
            )
        }
        Node::Str(s, _qt, style) => {
            let key_string = if matches!(style, BlockStyle::Literal) {
                format!("{}\n", s)
            } else {
                s.clone()
            };
            Node::Str(
                key_string,
                crate::nodes::node::QuoteType::Double,
                BlockStyle::None,
            )
        }
        other => {
            let inline = node_to_inline_string(other);
            Node::Str(
                inline,
                crate::nodes::node::QuoteType::Double,
                BlockStyle::None,
            )
        }
    }
}
use crate::io::traits::ISource;
use crate::nodes::node::Node;

/// Parses a single explicit key from the source and normalizes it to a string node.
#[allow(dead_code)]
fn parse_and_normalize_explicit_key(source: &mut dyn ISource) -> Result<Node, String> {
    let directives_local = crate::parser::directives::DirectiveContext::new();
    {
        let mut stream =
            crate::parser::token_stream::TokenStream::new(source, &directives_local, false)?;
        stream.skip_whitespace_and_comments()?;
    }
    let mut stream =
        crate::parser::token_stream::TokenStream::new(source, &directives_local, false)?;
    let mut key_node = match stream.current() {
        Some(crate::parser::lexer::Token::Newline) | None => {
            if source.current() == Some('\n') {
                source.next();
            }
            return Ok(Node::None);
        }
        _ => crate::parser::document::tokens::value::parse_value_with_tokens(
            &mut stream,
            &directives_local,
            0,
        )?,
    };
    key_node = normalize_node_to_str(&key_node);
    Ok(key_node)
}


/// Parses multiple explicit keys and their values for mappings.
///
/// Handles the case where we have multiple consecutive lines starting with '?',
/// each followed by a value, and collects (key, value) pairs into a mapping node.
pub fn parse_multiple_explicit_keys(
    source: &mut dyn ISource,
    _indent_level: usize,
) -> Result<Node, String> {
    use crate::parser::directives::DirectiveContext;
    use crate::parser::token_stream::TokenStream;
    let mut pairs: Vec<(Node, Node)> = Vec::new();
    let directives_local = DirectiveContext::new();
    let mut stream = TokenStream::new(source, &directives_local, false)?;
    while matches!(
        stream.current(),
        Some(crate::parser::lexer::Token::QuestionMark)
    ) {
        // Parse explicit mapping entry (? key : value)
        let (key, value) = crate::parser::document::explicit_key::parse_explicit_mapping_entry(
            &mut stream,
            &directives_local,
        )?;
        pairs.push((key, value));
        stream.skip_whitespace_and_comments()?;
        // Only continue if next token is another explicit key at the same indent
        if !matches!(
            stream.current(),
            Some(crate::parser::lexer::Token::QuestionMark)
        ) {
            break;
        }
    }
    Ok(Node::Mapping(pairs))
}
// Helper functions for parsing explicit mapping keys (? indicator)

use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;

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

    // Parse the key (may be empty), then normalize to string
    let mut key_node = match stream.current() {
        Some(Token::Newline) => {
            stream.next()?;
            stream.skip_whitespace()?;
            // Parse document contents as key (empty explicit key)
            crate::parser::document::tokens::value::parse_value_with_tokens(stream, directives, 0)?
        }
        _ => {
            crate::parser::document::tokens::value::parse_value_with_tokens(stream, directives, 0)?
        }
    };
    // Normalize key_node to Node::Str if not already
    key_node = normalize_node_to_str(&key_node);

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
                    crate::parser::document::tokens::value::parse_value_with_tokens(
                        stream, directives, 0,
                    )?
                }
                _ => crate::parser::document::tokens::value::parse_value_with_tokens(
                    stream, directives, 0,
                )?,
            }
        }
        // No value indicator, key only
        _ => Node::None,
    };

    Ok((key_node, value_node))
}
