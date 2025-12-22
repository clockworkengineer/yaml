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
    depth: usize,
) -> Result<Node, String> {
    #[cfg(feature = "debug-trace")]
    log::debug!(
        "inline_tokens: start flow sequence at token = {:?}",
        stream.current()
    );
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
                if expect_item {
                    // Comma found when expecting an item: leading or double comma
                    return Err(syntax_error(
                        stream.source_mut(),
                        "Leading or double comma in flow sequence is not allowed",
                    ));
                }
                // Allow trailing comma: set to expect next item, but do not error
                // If immediately followed by ']', the loop will close cleanly
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
                let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                #[cfg(feature = "debug-trace")]
                log::debug!("inline_tokens: seq item -> {:?}", value);
                items.push(value);
                expect_item = false;
            }
        }
    }
    #[cfg(feature = "debug-trace")]
    log::debug!(
        "inline_tokens: end flow sequence with {} item(s)",
        items.len()
    );
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
    depth: usize,
    is_set: bool,
) -> Result<Node, String> {
    println!(
        "DEBUG: ENTER parse_inline_mapping_with_tokens, current token: {:?}",
        stream.current()
    );
    // Expect opening brace
    stream.expect(Token::FlowMappingStart)?;

    let mut pairs = Vec::new();
    let mut expect_entry = true; // After { or comma, we expect a key

    let mut iteration = 0;
    loop {
        iteration += 1;
        if iteration > 1000 {
            println!(
                "DEBUG: Exceeded 1000 iterations in parse_inline_mapping_with_tokens, possible infinite loop"
            );
            return Err(
                "Exceeded 1000 iterations in flow mapping parser (possible infinite loop)"
                    .to_string(),
            );
        }
        // Skip whitespace/comments
        stream.skip_whitespace_and_comments()?;

        println!(
            "DEBUG: Iteration {}, current token: {:?}",
            iteration,
            stream.current()
        );
        match stream.current() {
            Some(Token::FlowMappingEnd) => {
                // Closing brace - done
                stream.next()?;
                break;
            }
            Some(Token::Comma) => {
                // Allow trailing comma: set to expect next entry, but do not error
                // If immediately followed by '}', the loop will close cleanly
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

                // Progress check: record position before parsing key
                let before_key = stream.stream_position();
                println!("DEBUG: before_key position = {}", before_key);
                let key = parse_value_with_tokens(stream, directives, depth + 1)?;
                let after_key = stream.stream_position();
                println!("DEBUG: after_key position = {}", after_key);
                if before_key == after_key {
                    return Err(syntax_error(
                        stream.source_mut(),
                        "Parser did not advance when parsing key in flow mapping (possible malformed input)",
                    ));
                }

                // Skip whitespace
                stream.skip_whitespace_and_comments()?;

                // Debug: print current token before colon check
                println!("DEBUG: Before colon check, current token: {:?}", stream.current());
                // Ensure all comments and newlines are skipped before colon check
                stream.skip_whitespace_and_comments()?;
                // Check for double colon (YAML compliance: not allowed)
                if matches!(stream.current(), Some(Token::Colon)) {
                    // Peek ahead for another colon (without whitespace/comments)
                    stream.next()?;
                    stream.skip_whitespace_and_comments()?;
                    if matches!(stream.current(), Some(Token::Colon)) {
                        // Found double colon, which is not allowed in YAML 1.2 flow mappings
                        return Err(syntax_error(
                            stream.source_mut(),
                            "YAML 1.2 compliance error: Double colon (::) is not allowed as a key-value separator in flow mappings. Use a single colon only."
                        ));
                    }
                    // Progress check: record position before parsing value
                    let before_value = stream.stream_position();
                    println!("DEBUG: before_value position = {}", before_value);
                    // Skip whitespace
                    stream.skip_whitespace_and_comments()?;
                    let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                    let after_value = stream.stream_position();
                    println!("DEBUG: after_value position = {}", after_value);
                    if before_value == after_value {
                        return Err(syntax_error(
                            stream.source_mut(),
                            "Parser did not advance when parsing value in flow mapping (possible malformed input)",
                        ));
                    }
                    #[cfg(feature = "debug-trace")]
                    log::debug!("inline_tokens: map entry -> ({:?}, {:?})", key, value);

                    pairs.push((key, value));
                } else {
                    if is_set {
                        // For sets, allow key with no colon (value is None)
                        #[cfg(feature = "debug-trace")]
                        log::debug!("inline_tokens: set entry -> ({:?}, None)", key);
                        pairs.push((key, Node::None));
                    } else {
                        // If not a colon, this is invalid in a flow mapping (YAML 1.2)
                        println!("DEBUG: Expected colon, got: {:?}", stream.current());
                        return Err(syntax_error(
                            stream.source_mut(),
                            "Expected colon after key in flow mapping",
                        ));
                    }
                }
                expect_entry = false;
            }
        }
    }
    #[cfg(feature = "debug-trace")]
    log::debug!(
        "inline_tokens: end flow mapping with {} pair(s)",
        pairs.len()
    );
    if is_set {
        // For !!set, convert mapping pairs with Node::None values to Node::Set
        let mut set_items = Vec::new();
        for (key, value) in &pairs {
            if let Node::None = value {
                set_items.push(key.clone());
            } else {
                // If any value is not None, fallback to mapping for compatibility
                return Ok(Node::Mapping(pairs));
            }
        }
        Ok(Node::Set(set_items))
    } else {
        Ok(Node::Mapping(pairs))
    }
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
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_inline_sequence_with_tokens(&mut stream, &directives, 0).unwrap();

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
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_inline_sequence_with_tokens(&mut stream, &directives, 0).unwrap();

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
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_inline_sequence_with_tokens(&mut stream, &directives, 0).unwrap();

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
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_inline_mapping_with_tokens(&mut stream, &directives, 0, false).unwrap();

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
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_inline_mapping_with_tokens(&mut stream, &directives, 0, false).unwrap();

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
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_inline_mapping_with_tokens(&mut stream, &directives, 0, false).unwrap();

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
        let mut stream = TokenStream::new(&mut source, &directives, false).unwrap();

        let result = parse_inline_sequence_with_tokens(&mut stream, &directives, 0).unwrap();

        if let Node::Array(items) = result {
            assert_eq!(items.len(), 2);
            assert!(matches!(items[0], Node::Array(_)));
            assert!(matches!(items[1], Node::Array(_)));
        } else {
            panic!("Expected Array node");
        }
    }
}
