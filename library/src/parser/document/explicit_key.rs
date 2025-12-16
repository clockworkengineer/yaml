use crate::io::traits::ISource;
use crate::nodes::node::{ Node};

/// Parses a single explicit key from the source and normalizes it to a string node.
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
    use crate::nodes::node::BlockStyle;
    use crate::parser::document::helpers::node_to_inline_string;
    match key_node {
        Node::Array(_) | Node::Mapping(_) => {
            let inline = node_to_inline_string(&key_node);
            key_node = Node::Str(
                inline,
                crate::nodes::node::QuoteType::Double,
                BlockStyle::None,
            );
        }
        Node::Str(s, _qt, style) => {
            let key_string = if matches!(style, BlockStyle::Literal) {
                format!("{}\n", s)
            } else {
                s
            };
            key_node = Node::Str(
                key_string,
                crate::nodes::node::QuoteType::Double,
                BlockStyle::None,
            );
        }
        other => {
            let inline = node_to_inline_string(&other);
            key_node = Node::Str(
                inline,
                crate::nodes::node::QuoteType::Double,
                BlockStyle::None,
            );
        }
    }
    Ok(key_node)
}

/// Determines if the loop should continue for explicit keys.
fn should_continue_explicit_key_loop(source: &mut dyn ISource, indent_level: usize) -> bool {
    crate::utils::skip_whitespace_and_comments(source);
    let current_indent = source.get_current_indent_level();
    current_indent == indent_level && source.current() == Some('?')
}

/// Parses multiple explicit keys for sets or mappings.
///
/// Handles the case where we have multiple consecutive lines starting with '?'
/// which typically represents a set with explicit key syntax.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `indent_level` - The current indentation level for proper nesting
///
/// # Returns
///
/// Result containing a Mapping Node with null values, suitable for set conversion
pub fn parse_multiple_explicit_keys(
    source: &mut dyn ISource,
    indent_level: usize,
) -> Result<Node, String> {
    let mut pairs: Vec<(Node, Node)> = Vec::new();
    while source.current() == Some('?') {
        source.next();
        let key_node = parse_and_normalize_explicit_key(source)?;
        if let Node::None = key_node {
            continue;
        }
        pairs.push((key_node, Node::None));
        if !should_continue_explicit_key_loop(source, indent_level) {
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

    // Parse the key (may be empty)
    let key_node = match stream.current() {
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
