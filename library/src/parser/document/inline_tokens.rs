//! Token-based flow collection parsers
//!
//! Handles inline YAML collections using tokens instead of character parsing.
//! This approach provides clearer boundaries and better error handling.

use crate::nodes::node::Node;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::error_builder::syntax_error;
use crate::parser::document::tokens::value::parse_value_with_tokens;
use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;

/// Parse a flow (inline) sequence using tokens
///
/// Example: `[1, 2, 3]` or `[a, b, c]`
///
/// Handles:
/// - Empty sequences: `[]`
/// - Trailing commas: `[1, 2, ]`
/// - Nested collections: `[[1, 2], [3, 4]]`
/// - Mixed types: `[1, "str", true]`
pub fn parse_inline_sequence_with_tokens(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    // Expect opening bracket
    stream.expect(Token::FlowSequenceStart)?;

    let mut items = Vec::new();
    let mut expect_item = true; // After [ or comma, we expect an item

    loop {
        // Skip whitespace/comments
        stream.skip_whitespace_and_comments()?;

        match stream.current() {
            Some(Token::FlowSequenceEnd) => {
                // Closing bracket - done
                stream.next()?;
                break;
            }
            Some(Token::Comma) => {
                if expect_item && !items.is_empty() {
                    // Comma after comma or at start (invalid)
                    return Err(syntax_error(
                        stream.source_mut(),
                        "Unexpected comma in flow sequence",
                    ));
                }
                stream.next()?;
                expect_item = true;
            }
            None | Some(Token::Eof) => {
                return Err(syntax_error(
                    stream.source_mut(),
                    "Unexpected end of input in flow sequence",
                ));
            }
            _ => {
                if !expect_item {
                    // Found value without comma separator
                    return Err(syntax_error(
                        stream.source_mut(),
                        "Expected comma or ] in flow sequence",
                    ));
                }

                // Parse the value
                let value = parse_value_with_tokens(stream, directives)?;
                items.push(value);
                expect_item = false;
            }
        }
    }

    Ok(Node::Array(items))
}

/// Parse a flow (inline) mapping using tokens
///
/// Example: `{a: 1, b: 2}` or `{key: value}`
///
/// Handles:
/// - Empty mappings: `{}`
/// - Trailing commas: `{a: 1, b: 2, }`
/// - Nested collections: `{a: {b: c}}`
/// - Quoted keys: `{"key": value}`
pub fn parse_inline_mapping_with_tokens(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    // Expect opening brace
    stream.expect(Token::FlowMappingStart)?;

    let mut pairs = Vec::new();
    let mut expect_entry = true; // After { or comma, we expect a key

    loop {
        // Skip whitespace/comments
        stream.skip_whitespace_and_comments()?;

        match stream.current() {
            Some(Token::FlowMappingEnd) => {
                // Closing brace - done
                stream.next()?;
                break;
            }
            Some(Token::Comma) => {
                if expect_entry && !pairs.is_empty() {
                    // Comma after comma or at start (invalid)
                    return Err(syntax_error(
                        stream.source_mut(),
                        "Unexpected comma in flow mapping",
                    ));
                }
                stream.next()?;
                expect_entry = true;
            }
            None | Some(Token::Eof) => {
                return Err(syntax_error(
                    stream.source_mut(),
                    "Unexpected end of input in flow mapping",
                ));
            }
            _ => {
                if !expect_entry {
                    // Found key-value without comma separator
                    return Err(syntax_error(
                        stream.source_mut(),
                        "Expected comma or } in flow mapping",
                    ));
                }

                // Parse the key
                let key = parse_value_with_tokens(stream, directives)?;

                // Skip whitespace
                stream.skip_whitespace_and_comments()?;

                // Expect colon
                if !matches!(stream.current(), Some(Token::Colon)) {
                    return Err(syntax_error(
                        stream.source_mut(),
                        "Expected : after mapping key in flow mapping",
                    ));
                }
                stream.next()?;

                // Skip whitespace
                stream.skip_whitespace_and_comments()?;

                // Parse the value
                let value = parse_value_with_tokens(stream, directives)?;

                pairs.push((key, value));
                expect_entry = false;
            }
        }
    }

    Ok(Node::Mapping(pairs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::parser::directives::DirectiveContext;

    #[test]
    fn test_empty_flow_sequence() {
        let yaml = b"[]";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_inline_sequence_with_tokens(&mut stream, &directives).unwrap();

        if let Node::Array(items) = result {
            assert_eq!(items.len(), 0);
        } else {
            panic!("Expected Array node");
        }
    }

    #[test]
    fn test_simple_flow_sequence() {
        let yaml = b"[1, 2, 3]";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_inline_sequence_with_tokens(&mut stream, &directives).unwrap();

        if let Node::Array(items) = result {
            assert_eq!(items.len(), 3);
        } else {
            panic!("Expected Array node");
        }
    }

    #[test]
    fn test_flow_sequence_trailing_comma() {
        let yaml = b"[1, 2, ]";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_inline_sequence_with_tokens(&mut stream, &directives).unwrap();

        if let Node::Array(items) = result {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected Array node");
        }
    }

    #[test]
    fn test_empty_flow_mapping() {
        let yaml = b"{}";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_inline_mapping_with_tokens(&mut stream, &directives).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 0);
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn test_simple_flow_mapping() {
        let yaml = b"{a: 1, b: 2}";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_inline_mapping_with_tokens(&mut stream, &directives).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn test_flow_mapping_trailing_comma() {
        let yaml = b"{a: 1, b: 2, }";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_inline_mapping_with_tokens(&mut stream, &directives).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
        } else {
            panic!("Expected Mapping node");
        }
    }

    #[test]
    fn test_nested_flow_collections() {
        let yaml = b"[[1, 2], [3, 4]]";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_inline_sequence_with_tokens(&mut stream, &directives).unwrap();

        if let Node::Array(items) = result {
            assert_eq!(items.len(), 2);
            assert!(matches!(items[0], Node::Array(_)));
            assert!(matches!(items[1], Node::Array(_)));
        } else {
            panic!("Expected Array node");
        }
    }
}
