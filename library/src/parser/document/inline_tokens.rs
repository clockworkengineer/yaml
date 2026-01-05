/// Token-based flow collection parsers
///
/// Handles inline YAML collections using tokens instead of character parsing.
/// This approach provides clearer boundaries and better error handling.
use crate::parser::document::node_utils::make_set_node;

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
                let _ = stream.consume_if(Token::FlowSequenceEnd)?;
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
                let _ = stream.consume_if(Token::Comma)?;
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

                // Parse what might be a value or the key of an implicit mapping
                let value_or_key = parse_value_with_tokens(stream, directives, depth + 1)?;

                // Skip whitespace to check if this is actually a key (followed by colon)
                stream.skip_trivia()?;

                // Check if this is an implicit mapping (key: value in a flow sequence)
                if matches!(stream.current(), Some(Token::Colon)) {
                    // This is actually a key, not a standalone value
                    // Parse as a single-pair mapping
                    let _ = stream.consume_if(Token::Colon)?; // consume colon
                    stream.skip_trivia()?;

                    // Parse the value (or use None if followed by comma/bracket)
                    let val = if matches!(
                        stream.current(),
                        Some(Token::Comma) | Some(Token::FlowSequenceEnd)
                    ) {
                        Node::None
                    } else {
                        parse_value_with_tokens(stream, directives, depth + 1)?
                    };

                    // Create a single-pair mapping
                    let mapping = Node::Mapping(vec![(value_or_key, val)]);
                    #[cfg(feature = "debug-trace")]
                    log::debug!(
                        "inline_tokens: seq item (implicit mapping) -> {:?}",
                        mapping
                    );
                    items.push(mapping);
                } else {
                    // It's a regular value
                    #[cfg(feature = "debug-trace")]
                    log::debug!("inline_tokens: seq item -> {:?}", value_or_key);
                    items.push(value_or_key);
                }
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
            return Err(syntax_error(
                stream.source_mut(),
                "Exceeded 1000 iterations in flow mapping parser (possible infinite loop)",
            ));
        }
        // Skip whitespace/comments
        stream.skip_trivia()?;

        println!(
            "DEBUG: Iteration {}, current token: {:?}",
            iteration,
            stream.current()
        );
        match stream.current() {
            Some(Token::FlowMappingEnd) => {
                // Closing brace - done
                let _ = stream.consume_if(Token::FlowMappingEnd)?;
                break;
            }
            Some(Token::Comma) => {
                // Allow trailing comma: set to expect next entry, but do not error
                // If immediately followed by '}', the loop will close cleanly
                let _ = stream.consume_if(Token::Comma)?;
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
                ensure_progress(stream, before_key, after_key, "key in flow mapping")?;

                // Skip whitespace
                stream.skip_trivia()?;

                // Debug: print current token before colon check
                println!(
                    "DEBUG: Before colon check, current token: {:?}",
                    stream.current()
                );
                // Ensure all comments and newlines are skipped before colon check
                stream.skip_trivia()?;
                // Expect colon for key-value pair
                if matches!(stream.current(), Some(Token::Colon)) {
                    // DRY: consume single colon with compliance validation (no behavior change)
                    let _ = stream.consume_single_colon()?;
                    stream.skip_trivia()?;

                    // Progress check: record position before parsing value
                    let before_value = stream.stream_position();
                    println!("DEBUG: before_value position = {}", before_value);

                    // Check for empty value (key: followed by , or })
                    let value = if matches!(
                        stream.current(),
                        Some(Token::Comma) | Some(Token::FlowMappingEnd)
                    ) {
                        // Empty value - use None (null)
                        Node::None
                    } else {
                        let val = parse_value_with_tokens(stream, directives, depth + 1)?;
                        let after_value = stream.stream_position();
                        println!("DEBUG: after_value position = {}", after_value);
                        ensure_progress(
                            stream,
                            before_value,
                            after_value,
                            "value in flow mapping",
                        )?;
                        val
                    };
                    #[cfg(feature = "debug-trace")]
                    log::debug!("inline_tokens: map entry -> ({:?}, {:?})", key, value);

                    pairs.push((key, value));
                } else {
                    // In flow mappings, a key without a colon has an implicit null value
                    // This is valid in YAML 1.2: {key} is equivalent to {key: null}
                    #[cfg(feature = "debug-trace")]
                    log::debug!(
                        "inline_tokens: map entry with implicit null -> ({:?}, None)",
                        key
                    );
                    pairs.push((key, Node::None));
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
        Ok(make_set_node(set_items))
    } else {
        Ok(Node::Mapping(pairs))
    }
}

/// Ensure that the token stream progressed between two checkpoints, else raise a syntax error.
fn ensure_progress(
    stream: &mut TokenStream,
    before: usize,
    after: usize,
    context: &str,
) -> Result<(), String> {
    if before == after {
        return Err(syntax_error(
            stream.source_mut(),
            &format!(
                "Parser did not advance when parsing {} (possible malformed input)",
                context
            ),
        ));
    }
    Ok(())
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
