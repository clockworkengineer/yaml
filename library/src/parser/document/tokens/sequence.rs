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
use crate::parser::document::context::ParsingContext;

pub fn parse_sequence_with_tokens(
    stream: &mut TokenStream,
    base_indent: usize,
    directives: &DirectiveContext,
    ctx: &ParsingContext,
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
    let mut stack: Vec<(usize, Vec<Node>)> = Vec::new();
    stack.push((base_indent, Vec::new()));

    // Skip initial trivia (whitespace, comments)
    stream.skip_trivia()?;

    loop {
        // Skip comments and newlines between sequence items
        // DO NOT skip Indent tokens here - we need them for dedent detection
        stream.skip_newlines_and_comments()?;
        // If a stray comma remains after an inline flow item, consume it
        if matches!(stream.current(), Some(Token::Comma)) {
            let _ = stream.consume_comma()?;
            // If immediately followed by a flow closer, consume it
            if matches!(stream.current(), Some(Token::FlowMappingEnd)) {
                let _ = stream.consume_flow_mapping_end()?;
            } else if matches!(stream.current(), Some(Token::FlowSequenceEnd)) {
                let _ = stream.consume_flow_sequence_end()?;
            }
            // Don't skip Indent tokens - we need them for dedent detection
            stream.skip_newlines_and_comments()?;
        }
        // End sequence if dedent or document marker, but only if not in explicit key or flow context
        let current_indent = stack.last().map(|(lvl, _)| *lvl).unwrap_or(base_indent);
        if !ctx.in_flow
            && !matches!(
                ctx.collection_type,
                crate::parser::document::context::CollectionType::BlockMapping
            )
        {
            match stream.current() {
                Some(Token::DocumentStart) => {
                    let (_, items) = stack.pop().unwrap();
                    if items.is_empty() {
                        return Ok(Node::None);
                    } else {
                        return Ok(Node::Array(items));
                    }
                }
                Some(Token::DocumentEnd) => {
                    // Document end marker - validate no content after it on same line
                    // The lexer has already consumed "..." and positioned us right after it
                    // Check for invalid content after ... on same line (before newline)
                    loop {
                        match stream.source_mut().current() {
                            Some(' ') | Some('\t') => stream.source_mut().next(),
                            Some('#') => {
                                while let Some(c) = stream.source_mut().current() {
                                    if c == '\n' || c == '\r' {
                                        break;
                                    }
                                    stream.source_mut().next();
                                }
                                break;
                            }
                            Some('\n') | Some('\r') | None => break,
                            Some(c) => {
                                return Err(format!(
                                    "Invalid content '{}' after document end marker (...)",
                                    c
                                ));
                            }
                        }
                    }
                    let (_, items) = stack.pop().unwrap();
                    if items.is_empty() {
                        return Ok(Node::None);
                    } else {
                        return Ok(Node::Array(items));
                    }
                }
                Some(Token::Indent(level)) if *level < current_indent => {
                    let (_, items) = stack.pop().unwrap();
                    return Ok(Node::Array(items));
                }
                _ => {}
            }
        }

        // Only parse a sequence item if the current token is a dash at the current indent
        let is_dash = matches!(stream.current(), Some(Token::Dash));
        if is_dash {
            stream.next()?;
            // Skip whitespace after dash, but preserve Indent tokens for dedent detection
            // Only skip newlines and comments here
            stream.skip_newlines_and_comments()?;
            match stream.current() {
                Some(Token::Newline) | None => {
                    if let Some((_, items)) = stack.last_mut() {
                        items.push(Node::None);
                    }
                    if let Some(Token::Newline) = stream.current() {
                        stream.next()?;
                    }
                }
                Some(Token::Dash) => {
                    // Nested sequence starts immediately (- - case)
                    // Use current_indent + 1 as base to properly detect dedents back to current level
                    let nested_base = current_indent + 1;
                    let ctx_seq = ctx.child_block_context(
                        nested_base,
                        crate::parser::document::context::CollectionType::BlockSequence,
                    );
                    let seq = parse_sequence_with_tokens(
                        stream,
                        nested_base,
                        directives,
                        &ctx_seq,
                        depth + 1,
                    )?;
                    if let Some((_, items)) = stack.last_mut() {
                        items.push(seq);
                    }
                }
                Some(Token::Indent(level)) => {
                    let indent = *level;
                    stream.next()?;
                    if indent > current_indent && matches!(stream.current(), Some(Token::Dash)) {
                        // New nested sequence: push to stack
                        stack.push((indent, Vec::new()));
                        continue;
                    } else if indent >= current_indent {
                        use crate::parser::document::tokens::mapping::parse_mapping_with_tokens;
                        let mapping =
                            parse_mapping_with_tokens(stream, indent, directives, depth + 1)?;
                        if let Some((_, items)) = stack.last_mut() {
                            items.push(mapping);
                        }
                    } else {
                        // Dedent: close current sequence
                        let (_, items) = stack.pop().unwrap();
                        if let Some((_, parent_items)) = stack.last_mut() {
                            parent_items.push(Node::Array(items));
                        } else {
                            return Ok(Node::Array(items));
                        }
                        continue;
                    }
                    stream.skip_newlines_and_comments()?;
                }
                Some(Token::FlowSequenceStart) | Some(Token::FlowMappingStart) => {
                    let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                    if let Some((_, items)) = stack.last_mut() {
                        items.push(value);
                    }
                    loop {
                        // Don't skip Indent tokens - needed for dedent detection
                        stream.skip_newlines_and_comments()?;
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
                    // Preserve behavior: only skip newlines here
                    while matches!(stream.current(), Some(Token::Newline)) {
                        stream.next()?;
                    }
                }
                Some(Token::Plain(_)) => {
                    // Don't skip indent tokens here - we need them for dedent detection
                    stream.skip_newlines_and_comments()?;
                    let mut is_colon = false;
                    if let Some(Token::Plain(_)) = stream.current() {
                        if let Some(Token::Colon) = stream.peek()? {
                            is_colon = true;
                        }
                    }
                    if is_colon {
                        use crate::parser::document::tokens::mapping::parse_mapping_with_tokens;
                        let indent = current_indent;
                        let mapping =
                            parse_mapping_with_tokens(stream, indent, directives, depth + 1)?;
                        if let Some((_, items)) = stack.last_mut() {
                            items.push(mapping);
                        }
                        // Don't skip Indent tokens - needed for dedent detection
                        stream.skip_newlines_and_comments()?;
                        if matches!(stream.current(), Some(Token::Comma)) {
                            stream.next()?;
                            if matches!(
                                stream.current(),
                                Some(Token::FlowMappingEnd) | Some(Token::FlowSequenceEnd)
                            ) {
                                stream.next()?;
                            }
                            // Don't skip Indent tokens - needed for dedent detection
                            stream.skip_newlines_and_comments()?;
                        }
                        stream.skip_newlines_and_comments()?;
                    } else {
                        let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                        if let Some((_, items)) = stack.last_mut() {
                            items.push(value);
                        }
                    }
                }
                _ => {
                    let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                    if let Some((_, items)) = stack.last_mut() {
                        items.push(value);
                    }
                }
            }

            // After parsing an item, skip whitespace/comments and check for another dash at the same indent
            // Skip newlines and comments, but NOT indents (we need to check indent level)
            stream.skip_newlines_and_comments()?;

            // Check if there's another dash at the current indent level
            match stream.current() {
                Some(Token::Dash) => {
                    // Dash at same indent, continue parsing
                    continue;
                }
                Some(Token::Indent(level)) if *level < current_indent => {
                    // Dedent detected, break out to return
                    break;
                }
                Some(Token::Indent(level)) if *level == current_indent => {
                    // Check if there's a dash after this indent
                    stream.next()?;
                    if matches!(stream.current(), Some(Token::Dash)) {
                        continue;
                    } else {
                        break;
                    }
                }
                _ => {
                    // No more items at this level
                    break;
                }
            }
        } else {
            // If not a dash, skip to next token or break if at end
            break;
        }
    }

    // Should not reach here, but return top-level sequence if stack not empty
    let (_, items) = stack.pop().unwrap_or((base_indent, Vec::new()));
    Ok(Node::Array(items))
}
