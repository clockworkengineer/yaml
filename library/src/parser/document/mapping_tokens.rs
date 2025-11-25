//! Token-based mapping parser
//!
//! Parses YAML mappings using tokenization instead of character-based lookahead.
//! This eliminates infinite loops with decorators and handles complex key patterns.

use crate::nodes::node::Node;
use crate::parser::token_stream::TokenStream;
use crate::parser::lexer::Token;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::tokens::value::parse_value_with_tokens;

/// Parse a block mapping using tokens
///
/// Example:
/// ```yaml
/// key1: value1
/// key2: value2
/// !!str: tagged_key
/// ? complex_key
/// : complex_value
/// ```
///
/// Benefits of token-based approach:
/// - No complex lookahead for keys with decorators
/// - Clear token boundaries prevent infinite loops
/// - Natural handling of explicit keys (?)
#[allow(dead_code)]
pub fn parse_mapping_with_tokens(
    stream: &mut TokenStream,
    _base_indent: usize,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    let mut pairs = Vec::new();

    // Skip initial whitespace/newlines
    stream.skip_whitespace()?;

    loop {
        // Check what we're looking at
        match stream.current() {
            Some(Token::Newline) => {
                // Skip empty lines
                stream.next()?;
                continue;
            }
            Some(Token::Indent(_level)) => {
                // Check indentation - consume for now
                stream.next()?;
                continue;
            }
            None | Some(Token::Eof) | Some(Token::DocumentEnd) | Some(Token::DocumentStart) => {
                // End of mapping
                break;
            }
            Some(Token::Dash) => {
                // Dash means we're in a sequence context, not mapping
                break;
            }
            Some(Token::FlowMappingEnd) | Some(Token::FlowSequenceEnd) => {
                // End of flow collection
                break;
            }
            _ => {
                // Parse a key-value pair
                let (key, value) = parse_mapping_pair(stream, directives)?;
                pairs.push((key, value));
                
                // Skip trailing whitespace
                stream.skip_whitespace()?;
            }
        }
    }

    Ok(Node::Mapping(pairs))
}

/// Parse a single key-value pair
#[allow(dead_code)]
fn parse_mapping_pair(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
) -> Result<(Node, Node), String> {
    // Check for explicit key indicator (?)
    let explicit_key = if matches!(stream.current(), Some(Token::Plain(s)) if s == "?") {
        stream.next()?;
        stream.skip_whitespace()?;
        true
    } else {
        false
    };

    // Check if we have decorators on an implicit empty key (decorator followed by colon)
    let key = if matches!(stream.current(), Some(Token::Tag(_)) | Some(Token::Anchor(_))) {
        // Consume decorators
        let decorators = stream.consume_decorators()?;
        
        // Check if followed immediately by colon (implicit empty key)
        if matches!(stream.current(), Some(Token::Colon)) {
            use crate::nodes::node::{QuoteType, BlockStyle};
            let mut node = Node::Str("".to_string(), QuoteType::Unquoted, BlockStyle::None);
            if let Some(tag) = decorators.tag {
                node = Node::Tagged(Box::new(node), tag);
            }
            if let Some(anchor) = decorators.anchor {
                node = Node::Anchored(Box::new(node), anchor);
            }
            node
        } else {
            // Decorators on actual key value - parse it
            parse_value_with_tokens(stream, directives)?
        }
    } else {
        // No decorators, parse key normally
        parse_value_with_tokens(stream, directives)?
    };
    
    // Skip whitespace after key
    stream.skip_whitespace()?;

    // Expect colon
    match stream.current() {
        Some(Token::Colon) => {
            stream.next()?;
        }
        _ if explicit_key => {
            // Explicit key might have colon on next line
            if matches!(stream.current(), Some(Token::Newline)) {
                stream.next()?;
                stream.skip_whitespace()?;
                if matches!(stream.current(), Some(Token::Colon)) {
                    stream.next()?;
                } else {
                    return Err("Expected colon after explicit key".to_string());
                }
            } else {
                return Err("Expected colon after explicit key".to_string());
            }
        }
        _ => {
            return Err(format!("Expected colon after key, got: {:?}", stream.current()));
        }
    }

    // Parse the value - check for empty value BEFORE skipping whitespace
    let value = match stream.current() {
        Some(Token::Newline) | None | Some(Token::Eof) => {
            // Empty value - don't consume the newline/eof
            Node::None
        }
        _ => {
            // Skip whitespace before value
            stream.skip_whitespace()?;
            // Parse the actual value
            parse_value_with_tokens(stream, directives)?
        }
    };

    Ok((key, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::parser::directives::DirectiveContext;

    #[test]
    fn test_simple_mapping() {
        let yaml = b"key1: value1\nkey2: value2";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn test_mapping_with_empty_value() {
        let yaml = b"key1:\nkey2: value2";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(pairs[0].1, Node::None));
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn test_mapping_with_decorated_key() {
        let yaml = b"!!str: value\n&anchor: value2";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // First key should be tagged empty string
            // Second key should be anchored
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn test_fh7j_nested_mapping() {
        // FH7J has: "  !!null : a\n  b: !!str\n"
        let yaml = b"!!null: a\nb: !!str";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // First key should be tagged null (empty)
            // Second value should be tagged empty string
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }
}
