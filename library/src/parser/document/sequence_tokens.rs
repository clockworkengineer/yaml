//! Token-based sequence parser
//!
//! Parses YAML sequences using tokenization instead of character-based lookahead.
//! This eliminates infinite loops with decorators and simplifies the logic.

use crate::nodes::node::Node;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::tokens::value::parse_value_with_tokens;
use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;

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
    base_indent: usize,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    let mut items = Vec::new();

    // Skip initial whitespace/newlines but track where we start
    stream.skip_whitespace()?;

    loop {
        // Check for indentation change that would end the sequence
        if let Some(Token::Indent(level)) = stream.current() {
            if *level < base_indent {
                // Dedent - sequence is done
                break;
            }
            stream.next()?;
            continue;
        }

        // Check if we're at a dash (sequence indicator)
        match stream.current() {
            Some(Token::Dash) => {
                // Consume the dash
                stream.next()?;

                // Skip whitespace after dash
                stream.skip_whitespace()?;

                // Check what follows the dash
                // ...existing code...
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
                    Some(Token::Indent(_)) => {
                        // Indented block after dash: always parse as mapping
                        use crate::parser::document::mapping_tokens::parse_mapping_with_tokens;
                        let indent = match stream.current() {
                            Some(Token::Indent(level)) => *level,
                            _ => 0,
                        };
                        let mapping = parse_mapping_with_tokens(stream, indent, directives)?;
                        items.push(mapping);
                        // Skip trailing whitespace/newlines until we see next dash or end
                        loop {
                            match stream.current() {
                                Some(Token::Newline) => {
                                    stream.next()?;
                                }
                                Some(Token::Indent(_)) | Some(Token::Dash) | None => {
                                    break;
                                }
                                _ => break,
                            }
                        }
                    }
                    Some(Token::Plain(_)) => {
                        // Check for mapping pattern: plain token followed by colon
                        // Skip whitespace/comments after dash
                        stream.skip_whitespace_and_comments()?;
                        let mut is_colon = false;
                        if let Some(Token::Plain(_)) = stream.current() {
                            // Peek ahead for colon without advancing
                            if let Some(Token::Colon) = stream.peek()? {
                                is_colon = true;
                            }
                        }

                        if is_colon {
                            use crate::parser::document::mapping_tokens::parse_mapping_with_tokens;
                            let indent = base_indent;
                            let mapping = parse_mapping_with_tokens(stream, indent, directives)?;
                            items.push(mapping);
                            // Skip trailing whitespace/newlines until we see next dash or end
                            loop {
                                match stream.current() {
                                    Some(Token::Newline) => {
                                        stream.next()?;
                                    }
                                    Some(Token::Indent(_)) | Some(Token::Dash) | None => {
                                        break;
                                    }
                                    _ => break,
                                }
                            }
                        } else {
                            println!(
                                "DEBUG: sequence_tokens: Parsing value after dash (not mapping)"
                            );
                            let value = parse_value_with_tokens(stream, directives)?;
                            println!("DEBUG: sequence_tokens: value node = {:?}", value);
                            items.push(value);
                            // Skip trailing whitespace/newlines until we see next dash or end
                            loop {
                                match stream.current() {
                                    Some(Token::Newline) => {
                                        stream.next()?;
                                    }
                                    Some(Token::Indent(_)) | Some(Token::Dash) | None => {
                                        break;
                                    }
                                    _ => break,
                                }
                            }
                        }
                    }
                    _ => {
                        // Parse the value
                        let value = parse_value_with_tokens(stream, directives)?;
                        items.push(value);

                        // Skip trailing whitespace/newlines until we see next dash or end
                        loop {
                            match stream.current() {
                                Some(Token::Newline) => {
                                    stream.next()?;
                                }
                                Some(Token::Indent(_)) | Some(Token::Dash) | None => {
                                    break;
                                }
                                _ => break,
                            }
                        }
                    }
                }
            }
            Some(Token::Newline) => {
                // Skip empty lines
                stream.next()?;
            }
            None | Some(Token::DocumentEnd) | Some(Token::DocumentStart) | Some(Token::Eof) => {
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
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

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
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

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
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

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
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_sequence_with_tokens(&mut stream, 0, &directives).unwrap();

        if let Node::Array(items) = result {
            assert_eq!(items.len(), 1);
            // Should be tagged empty string (!!str on empty value)
            // Coercion produces Str, but non-coercible tags are stored as Tagged
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
