/// Parse a single key-value mapping pair (for sequence items)
#[allow(dead_code)]
pub fn parse_single_mapping_pair_with_tokens(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    let (key, value) = parse_mapping_pair(stream, directives)?;
    Ok(Node::Mapping(vec![(key, value)]))
}
// Token-based mapping parser: Parses YAML mappings using tokenization instead of character-based lookahead. This eliminates infinite loops with decorators and handles complex key patterns.

use crate::nodes::node::Node;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::tokens::value::parse_value_with_tokens;
use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;

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
    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_tokens: start parse_mapping_with_tokens");
    let mut pairs = Vec::new();
    // ...existing code...

    // Skip initial whitespace/newlines
    stream.skip_whitespace()?;

    // Track current indentation level
    let mut current_indent: Option<usize> = None;

    while let Some(token) = stream.current() {
        match token {
            Token::Newline | Token::Comment(_) => {
                stream.next()?;
                continue;
            }
            Token::Indent(level) => {
                if current_indent.is_none() {
                    current_indent = Some(*level);
                    stream.next()?;
                    continue;
                } else if let Some(cur) = current_indent {
                    if *level < cur {
                        break;
                    } else if *level > cur {
                        let nested = parse_mapping_with_tokens(stream, *level, directives)?;
                        if let Some((last_key, last_value)) = pairs.pop() {
                            let new_value = match last_value {
                                Node::None => nested,
                                _ => last_value,
                            };
                            pairs.push((last_key, new_value));
                        }
                        continue;
                    } else {
                        stream.next()?;
                        continue;
                    }
                }
            }
            Token::Eof
            | Token::DocumentEnd
            | Token::DocumentStart
            | Token::Dash
            | Token::FlowMappingEnd
            | Token::FlowSequenceEnd => {
                break;
            }
            _ => {
                let (key, value) = parse_mapping_pair(stream, directives)?;
                // Use token-based helpers for key/value safety (example usage below)
                // if let Some(Token::Plain(ref k)) = stream.current() {
                //     let key_is_safe = is_plain_safe_key_token(token);
                // }
                pairs.push((key, value));
                stream.skip_whitespace()?;
            }
        }
    }

    #[cfg(feature = "debug-trace")]
    log::debug!(
        "mapping_tokens: end parse_mapping_with_tokens with {} pair(s)",
        pairs.len()
    );
    Ok(Node::Mapping(pairs))
}

/// Parse a single key-value pair
#[allow(dead_code)]
fn parse_mapping_pair(
    stream: &mut TokenStream,
    directives: &DirectiveContext,
) -> Result<(Node, Node), String> {
    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_pair: start at token = {:?}", stream.current());
    // Check for explicit key indicator (?)
    // Lexer emits a dedicated token for '?', not a plain scalar
    let explicit_key = if matches!(stream.current(), Some(Token::QuestionMark)) {
        stream.next()?;
        stream.skip_whitespace()?;
        true
    } else {
        false
    };

    // Check if we have decorators on an implicit empty key (decorator followed by colon)
    let key = if matches!(
        stream.current(),
        Some(Token::Tag(_)) | Some(Token::Anchor(_))
    ) {
        // Consume decorators
        let decorators = stream.consume_decorators()?;

        // Check if followed immediately by colon (implicit empty key)
        if matches!(stream.current(), Some(Token::Colon)) {
            use crate::nodes::node::{BlockStyle, QuoteType};
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
            let parsed_key = parse_value_with_tokens(stream, directives)?;
            #[cfg(feature = "debug-trace")]
            log::debug!("mapping_pair: parsed decorated key = {:?}", parsed_key);
            parsed_key
        }
    } else {
        // No decorators, parse key normally
        let parsed_key = parse_value_with_tokens(stream, directives)?;
        #[cfg(feature = "debug-trace")]
        log::debug!("mapping_pair: parsed key = {:?}", parsed_key);
        parsed_key
    };

    // Skip whitespace and comments after key
    loop {
        stream.skip_whitespace()?;
        match stream.current() {
            Some(Token::Comment(_)) => {
                stream.next()?;
                continue;
            }
            _ => break,
        }
    }

    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_pair: after key, token = {:?}", stream.current());
    // Expect colon, or treat as empty value if next token is a valid key
    match stream.current() {
        Some(Token::Colon) => {
            stream.next()?;
        }
        _ if explicit_key => {
            // Explicit key may omit a value entirely (e.g., !!set with '? key').
            // YAML allows explicit keys without a following colon to indicate an
            // empty value. If the colon isn't found on the next non-trivia token,
            // treat the value as empty rather than error.
            if matches!(stream.current(), Some(Token::Newline)) {
                stream.next()?;
                // Skip indentation and comments
                loop {
                    stream.skip_whitespace()?;
                    match stream.current() {
                        Some(Token::Comment(_)) => {
                            stream.next()?;
                            continue;
                        }
                        _ => break,
                    }
                }
                if matches!(stream.current(), Some(Token::Colon)) {
                    stream.next()?;
                } else {
                    // No colon: treat as empty value for explicit key
                    return Ok((key, Node::None));
                }
            } else {
                // No newline after explicit key: if colon absent, treat as empty
                if !matches!(stream.current(), Some(Token::Colon)) {
                    return Ok((key, Node::None));
                } else {
                    stream.next()?;
                }
            }
        }
        Some(Token::Eof) | None => {
            // Treat EOF or None as valid empty value
            return Ok((key, Node::None));
        }
        // If next token is a valid key, treat as empty value
        Some(Token::Plain(_))
        | Some(Token::Tag(_))
        | Some(Token::Anchor(_))
        | Some(Token::QuestionMark) => {
            return Ok((key, Node::None));
        }
        _ => {
            return Err(format!(
                "Expected colon after key, got: {:?}",
                stream.current()
            ));
        }
    }

    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_pair: before value, token = {:?}", stream.current());
    // Parse the value - check for empty value BEFORE skipping whitespace
    let value = match stream.current() {
        Some(Token::Newline) | None | Some(Token::Eof) => {
            // Empty value - don't consume the newline/eof
            Node::None
        }
        Some(Token::Indent(level)) => {
            // Increased indentation: parse nested mapping as value
            let nested = parse_mapping_with_tokens(stream, *level, directives)?;
            nested
        }
        _ => {
            // Skip whitespace before value
            stream.skip_whitespace()?;
            // Parse the actual value
            let v = parse_value_with_tokens(stream, directives)?;
            #[cfg(feature = "debug-trace")]
            log::debug!("mapping_pair: parsed value = {:?}", v);
            v
        }
    };
    #[cfg(feature = "debug-trace")]
    log::debug!("mapping_pair: return pair = ({:?}, {:?})", key, value);
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

    #[test]
    fn test_explicit_keys_block_mapping() {
        // Explicit keys without values should produce Node::None values
        let yaml = b"? item1\n? item2\n? item3\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 3);
            assert!(matches!(pairs[0].1, Node::None));
            assert!(matches!(pairs[1].1, Node::None));
            assert!(matches!(pairs[2].1, Node::None));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_explicit_key_with_value() {
        // Explicit key followed by value on same line
        let yaml = b"? key1: value1\n? key2\n: value2\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            // key1 has value1; key2 should have value2
            assert!(matches!(pairs[0].0, Node::Str(_, _, _)));
            assert!(matches!(pairs[0].1, Node::Str(ref s, _, _) if s == "value1"));
            assert!(matches!(pairs[1].0, Node::Str(_, _, _)));
            assert!(matches!(pairs[1].1, Node::Str(ref s, _, _) if s == "value2"));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_explicit_complex_key_array() {
        // Complex explicit key (array) should normalize to string key
        let yaml = b"? [a, b, c]: 1\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 1);
            // Key should be a string representation of the array
            assert!(
                matches!(pairs[0].0, Node::Str(ref s, _, _) if s.contains("a") && s.contains("b") && s.contains("c"))
            );
            assert!(matches!(
                pairs[0].1,
                Node::Number(crate::nodes::node::Numeric::Integer(1))
            ));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_empty_mapping() {
        let yaml = b"{}\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        // Inline empty mapping should parse via inline_tokens, but base parser should gracefully handle
        let node =
            super::super::inline_tokens::parse_inline_mapping_with_tokens(&mut stream, &directives)
                .unwrap();
        assert!(matches!(node, Node::Mapping(ref v) if v.is_empty()));
    }

    #[test]
    fn test_multiline_key_value_mapping() {
        // Multiline plain scalar key and value using block scalar-like lines
        let yaml = b"? |\n  multi\n  line\n: |\n  val\n  ue\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 1);
            // Keys/values produced by scalar parser should be strings (literal preserves newlines)
            assert!(
                matches!(pairs[0].0, Node::Str(ref s, _, _) if s.contains("multi") && s.contains("line"))
            );
            assert!(
                matches!(pairs[0].1, Node::Str(ref s, _, _) if s.contains("val") && s.contains("ue"))
            );
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }

    #[test]
    fn test_empty_value_on_same_line_and_next_line() {
        let yaml = b"key1: \nkey2:\n  - 1\n";
        let mut source = Buffer::new(yaml);
        let directives = DirectiveContext::new();
        let mut stream = TokenStream::new(&mut source, &directives).unwrap();

        let result = parse_mapping_with_tokens(&mut stream, 0, &directives).unwrap();

        if let Node::Mapping(pairs) = result {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(pairs[0].1, Node::None));
            assert!(matches!(pairs[1].1, Node::Array(_)));
        } else {
            panic!("Expected Mapping node, got: {:?}", result);
        }
    }
}
