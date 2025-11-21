//! Token-based sequence parser
//!
//! Parses YAML sequences using tokenization instead of character-based lookahead.
//! This eliminates infinite loops with decorators and simplifies the logic.

use crate::nodes::node::Node;
use crate::parser::token_stream::TokenStream;
use crate::parser::lexer::Token;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::value_tokens::parse_value_with_tokens;

/// Parse a block sequence using tokens
///
/// Example:
/// ```yaml
/// - item1
/// - item2
/// - !!str
/// - &anchor
/// ```
///
/// Benefits of token-based approach:
/// - No complex lookahead for decorators
/// - Clear token boundaries prevent infinite loops
/// - Natural handling of empty items after decorators
pub fn parse_sequence_with_tokens(
    stream: &mut TokenStream,
    _base_indent: usize,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    let mut items = Vec::new();

    // Skip initial whitespace/newlines
    stream.skip_whitespace()?;

    loop {
        // Check if we're at a dash (sequence indicator)
        match stream.current() {
            Some(Token::Dash) => {
                // Consume the dash
                stream.next()?;
                
                // Skip whitespace after dash
                stream.skip_whitespace()?;

                // Check what follows the dash
                match stream.current() {
                    Some(Token::Newline) | None => {
                        // Empty item (dash followed by newline or EOF)
                        items.push(Node::None);
                        if let Some(Token::Newline) = stream.current() {
                            stream.next()?;
                        }
                    }
                    Some(Token::Dash) => {
                        // Next item starts immediately (dash followed by dash)
                        items.push(Node::None);
                        // Don't consume - let next iteration handle it
                    }
                    _ => {
                        // Parse the value
                        let value = parse_value_with_tokens(stream, directives)?;
                        items.push(value);
                        
                        // Skip trailing whitespace
                        stream.skip_whitespace()?;
                    }
                }
            }
            Some(Token::Newline) => {
                // Skip empty lines
                stream.next()?;
            }
            Some(Token::Indent(level)) => {
                // Check indentation - if it's less than base, we're done
                // For now, just consume and continue
                stream.next()?;
            }
            None | Some(Token::DocumentEnd) | Some(Token::DocumentStart) => {
                // End of sequence
                break;
            }
            _ => {
                // Unexpected token - might be end of sequence
                break;
            }
        }
    }

    Ok(Node::Array(items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::parser::directives::DirectiveContext;

    #[test]
    fn test_simple_sequence() {
        let yaml = b"- a\n- b\n- c";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives);

        // Debug: print first few tokens
        println!("First token: {:?}", stream.current());
        
        let result = parse_sequence_with_tokens(&mut stream, 0, &directives).unwrap();

        if let Node::Array(items) = result {
            assert_eq!(items.len(), 3, "Expected 3 items, got: {:?}", items);
        } else {
            panic!("Expected Array node, got: {:?}", result);
        }
    }

    #[test]
    fn test_sequence_with_empty_item() {
        let yaml = b"- a\n-\n- c";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives);

        let result = parse_sequence_with_tokens(&mut stream, 0, &directives).unwrap();

        if let Node::Array(items) = result {
            assert_eq!(items.len(), 3);
            assert!(matches!(items[1], Node::None));
        } else {
            panic!("Expected Array node");
        }
    }

    #[test]
    fn test_sequence_with_decorator_on_empty() {
        let yaml = b"- !!str\n- &a\n- c";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives);

        let result = parse_sequence_with_tokens(&mut stream, 0, &directives).unwrap();

        if let Node::Array(items) = result {
            assert_eq!(items.len(), 3);
            // First item should be empty string (coerced from tagged empty)
            // Second item should be anchored empty
        } else {
            panic!("Expected Array node");
        }
    }

    #[test]
    fn test_fh7j_first_item() {
        // FH7J pattern: "- !!str\n"
        let yaml = b"- !!str\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives);

        let result = parse_sequence_with_tokens(&mut stream, 0, &directives).unwrap();

        if let Node::Array(items) = result {
            assert_eq!(items.len(), 1);
            // Should be tagged empty string (!!str on empty value)
            // Coercion produces Str, but non-coercible tags are stored as Tagged
            println!("Got item: {:?}", items[0]);
            match &items[0] {
                Node::Str(s, _, _) => assert!(s.is_empty(), "Expected empty string"),
                Node::Tagged(inner, tag) => {
                    assert!(tag.contains("str"), "Expected str tag");
                    assert!(matches!(inner.as_ref(), Node::Str(s, _, _) if s.is_empty()));
                }
                _ => panic!("Expected Str or Tagged(Str), got: {:?}", items[0]),
            }
        } else {
            panic!("Expected Array node, got: {:?}", result);
        }
    }
}
