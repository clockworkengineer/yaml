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
    depth: usize,
) -> Result<Node, String> {
    #[cfg(feature = "debug-trace")]
    log::debug!(
        "sequence_tokens: start parse_sequence_with_tokens at indent {}",
        base_indent
    );
    #[cfg(feature = "debug-trace")]
    log::debug!(
        "sequence_tokens: ENTER parse_sequence_with_tokens, indent={}, depth={}",
        base_indent,
        depth
    );
    let mut items = Vec::new();

    // Skip initial whitespace/newlines but track where we start
    stream.skip_whitespace()?;

    loop {
        println!(
            "DEBUG: [seq] depth={}, base_indent={}, current token={:?}",
            depth,
            base_indent,
            stream.current()
        );
        // Skip comments and newlines between sequence items
        while matches!(
            stream.current(),
            Some(Token::Comment(_)) | Some(Token::Newline)
        ) {
            stream.next()?;
        }
        // If a stray comma remains after an inline flow item, consume it
        if matches!(stream.current(), Some(Token::Comma)) {
            stream.next()?;
            // Also consume an optional closing flow token following it
            if matches!(
                stream.current(),
                Some(Token::FlowMappingEnd) | Some(Token::FlowSequenceEnd)
            ) {
                stream.next()?;
            }
            // Normalize whitespace/comments after consuming
            stream.skip_whitespace_and_comments()?;
        }
        // Check for document start/end marker at any depth
        match stream.current() {
            Some(Token::DocumentStart) | Some(Token::DocumentEnd) => {
                // Always break to main document loop, even if deeply nested
                // Only return Node::None if no items have been collected (empty sequence)
                if items.is_empty() {
                    return Ok(Node::None);
                } else {
                    break;
                }
            }
            _ => {}
        }
        // Check for indentation change that would end the sequence
        if let Some(Token::Indent(level)) = stream.current() {
            println!(
                "DEBUG: [seq] Indent token: level={}, base_indent={}",
                level, base_indent
            );
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
                println!(
                    "DEBUG: [seq] Got dash at depth={}, base_indent={}",
                    depth, base_indent
                );
                // Consume the dash
                stream.next()?;

                // Skip whitespace and comments after dash
                stream.skip_whitespace_and_comments()?;

                // Check what follows the dash
                match stream.current() {
                    Some(Token::Newline) | None => {
                        // Empty item (dash followed by newline or EOF)
                        println!("DEBUG: [seq] Dash followed by newline/EOF, push None");
                        items.push(Node::None);
                        if let Some(Token::Newline) = stream.current() {
                            stream.next()?;
                        }
                    }
                    Some(Token::Dash) => {
                        // Dash followed by dash at same indent: recurse as nested sequence
                        println!("DEBUG: [seq] Dash followed by dash, recurse as nested sequence");
                        let seq =
                            parse_sequence_with_tokens(stream, base_indent, directives, depth + 1)?;
                        items.push(seq);
                    }
                    Some(Token::Indent(level)) => {
                        // Indented block after dash: check if it's a sequence or mapping
                        let indent = *level;
                        println!(
                            "DEBUG: [seq] Dash followed by indent: {} > {}?",
                            indent, base_indent
                        );
                        stream.next()?; // consume Indent
                        // If indent is greater than base_indent and next token is dash, recurse as nested sequence
                        if indent > base_indent && matches!(stream.current(), Some(Token::Dash)) {
                            println!(
                                "DEBUG: [seq] Recursing into nested sequence at indent {}",
                                indent
                            );
                            let seq =
                                parse_sequence_with_tokens(stream, indent, directives, depth + 1)?;
                            items.push(seq);
                        } else if indent >= base_indent {
                            // Parse as mapping (or possibly flat sequence)
                            use crate::parser::document::tokens::mapping::parse_mapping_with_tokens;
                            let mapping =
                                parse_mapping_with_tokens(stream, indent, directives, depth + 1)?;
                            items.push(mapping);
                        } else {
                            // Dedent - sequence is done
                            println!("DEBUG: [seq] Dedent after dash+indent, breaking");
                            break;
                        }
                        // Skip trailing whitespace/comments/newlines until next dash or end
                        loop {
                            match stream.current() {
                                Some(Token::Newline) | Some(Token::Comment(_)) => {
                                    stream.next()?;
                                }
                                Some(Token::Indent(_)) | Some(Token::Dash) | None => {
                                    break;
                                }
                                _ => break,
                            }
                        }
                    }
                    // Standard-compliant: handle flow collections (empty or not) after dash at any nesting
                    Some(Token::FlowSequenceStart) | Some(Token::FlowMappingStart) => {
                        let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                        items.push(value);
                        // Normalize and aggressively consume separators/closers
                        loop {
                            stream.skip_whitespace_and_comments()?;
                            match stream.current() {
                                Some(Token::Comma) => {
                                    stream.next()?;
                                    continue;
                                }
                                Some(Token::FlowMappingEnd) | Some(Token::FlowSequenceEnd) => {
                                    stream.next()?;
                                    continue;
                                }
                                _ => break,
                            }
                        }
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
                            use crate::parser::document::tokens::mapping::parse_mapping_with_tokens;
                            let indent = base_indent;
                            let mapping =
                                parse_mapping_with_tokens(stream, indent, directives, depth + 1)?;
                            items.push(mapping);
                            // Normalize whitespace/comments after item
                            stream.skip_whitespace_and_comments()?;
                            // Guard: if a stray comma precedes a closing flow bracket across lines,
                            // consume it and the closing token to fully terminate the inline value.
                            if matches!(stream.current(), Some(Token::Comma)) {
                                stream.next()?;
                                if matches!(
                                    stream.current(),
                                    Some(Token::FlowMappingEnd) | Some(Token::FlowSequenceEnd)
                                ) {
                                    stream.next()?;
                                }
                                stream.skip_whitespace_and_comments()?;
                            }
                            // Skip trailing whitespace/comments/newlines until next dash or end
                            loop {
                                match stream.current() {
                                    Some(Token::Newline) | Some(Token::Comment(_)) => {
                                        stream.next()?;
                                    }
                                    Some(Token::Indent(_)) | Some(Token::Dash) | None => {
                                        break;
                                    }
                                    _ => break,
                                }
                            }
                        } else {
                            #[cfg(feature = "debug-trace")]
                            log::debug!("sequence_tokens: Parsing value after dash (not mapping)");
                            let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                            #[cfg(feature = "debug-trace")]
                            log::debug!("sequence_tokens: parsed value node = {:?}", value);
                            items.push(value);
                            // Normalize and aggressively consume separators/closers
                            loop {
                                stream.skip_whitespace_and_comments()?;
                                match stream.current() {
                                    Some(Token::Comma) => {
                                        stream.next()?;
                                        continue;
                                    }
                                    Some(Token::FlowMappingEnd) | Some(Token::FlowSequenceEnd) => {
                                        stream.next()?;
                                        continue;
                                    }
                                    _ => break,
                                }
                            }
                            // Skip trailing whitespace/newlines until next dash or end
                            loop {
                                match stream.current() {
                                    Some(Token::Newline) | Some(Token::Comment(_)) => {
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
                    other => {
                        println!("DEBUG: [seq] Unexpected token after dash: {:?}", other);
                        let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                        items.push(value);
                    }
                }
            }
            other => {
                println!("DEBUG: [seq] Non-dash token at start: {:?}", other);
                break;
            }
        }
    }

    #[cfg(feature = "debug-trace")]
    log::debug!(
        "sequence_tokens: end parse_sequence_with_tokens with {} item(s)",
        items.len()
    );
    Ok(Node::Array(items))
}
