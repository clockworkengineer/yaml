//! Sequence Token Parsing
//!
//! Implements token-based parsing for YAML sequences, eliminating infinite loops
//! with decorators and simplifying sequence parsing logic.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::nodes::node::Node;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::flow_punctuation;
use crate::parser::tokens::value::parse_value_with_tokens;
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
    parent_indent: usize,
    directives: &DirectiveContext,
    ctx: &ParsingContext,
    depth: usize,
) -> crate::parser::ParseResult<Node> {
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
    use crate::utils::optimization::{CapacityHints, NodeBuilder};
    // Use a small capacity profile for typical sequences
    let node_builder = NodeBuilder::with_hints(CapacityHints::small());
    let mut stack: Vec<(usize, Vec<Node>)> = Vec::new();
    // Pre-allocate sequence items using NodeBuilder
    stack.push((
        base_indent,
        Vec::with_capacity(node_builder.hints().sequence_items),
    ));

    // Skip initial trivia (whitespace, comments)
    stream.skip_trivia()?;

    loop {
        // Skip comments and newlines between sequence items
        // DO NOT skip Indent tokens here - we need them for dedent detection
        stream.skip_newlines_and_comments()?;
        // If a stray comma remains after an inline flow item, consume it
        if matches!(stream.current(), Some(Token::Comma)) {
            let _ = stream.consume_if(Token::Comma)?;
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
                    crate::parser::document::helpers::validate_trailing_content_after_document_end(
                        stream,
                    )?;
                    let (_, items) = stack.pop().unwrap();
                    if items.is_empty() {
                        return Ok(Node::None);
                    } else {
                        return Ok(Node::Array(items));
                    }
                }
                Some(Token::Indent(level)) if *level < current_indent => {
                    let dedent_level = *level;
                    // If the dedent takes us back to or above the parent
                    // indentation, treat it as the natural end of this
                    // sequence. Otherwise, we have an inconsistent
                    // indentation within the sequence body (e.g. 4HVU),
                    // which should be reported as an indentation error
                    // rather than silently starting a new structural block.
                    if dedent_level <= parent_indent {
                        let (_, items) = stack.pop().unwrap();
                        return Ok(Node::Array(items));
                    } else {
                        return Err(crate::parser::document::error_builder::indentation_error(
                            stream.source_mut(),
                            "Invalid indentation for sequence item",
                        ));
                    }
                }
                _ => {}
            }
        }

        // Only parse a sequence item if the current token is a dash at the current indent
        let is_dash = matches!(stream.current(), Some(Token::Dash));
        if is_dash {
            stream.next()?;
            // Check for empty sequence item (dash followed by newline or EOF)
            // Don't skip whitespace here - we need to check for newline first
            match stream.current() {
                Some(Token::Newline) => {
                    // We have a newline after the dash. Check if there's indented content after it.
                    stream.next()?; // Consume the newline
                    // Now check if there's an Indent token indicating nested content
                    match stream.current() {
                        Some(Token::Indent(level)) if *level > current_indent => {
                            // There's indented content after the dash - this is NOT an empty item
                            // Put the stream state back and fall through to parse the value
                            // Actually, we can't put it back, so just parse the indented content
                            let indent = *level;
                            stream.next()?;
                            if matches!(stream.current(), Some(Token::Dash)) {
                                // Nested sequence
                                stack.push((indent, Vec::new()));
                                continue;
                            } else {
                                // Parse as mapping or other value
                                use crate::parser::tokens::mapping::parse_mapping_with_tokens;
                                let mapping = parse_mapping_with_tokens(
                                    stream,
                                    indent,
                                    directives,
                                    depth + 1,
                                )?;
                                if let Some((_, items)) = stack.last_mut() {
                                    items.push(mapping);
                                }
                            }
                        }
                        _ => {
                            // No indented content after the newline - this IS an empty item
                            if let Some((_, items)) = stack.last_mut() {
                                items.push(Node::None);
                            }
                            // Continue to next iteration after handling empty item
                            continue;
                        }
                    }
                }
                None => {
                    // EOF after dash - empty item
                    if let Some((_, items)) = stack.last_mut() {
                        items.push(Node::None);
                    }
                    continue;
                }
                Some(Token::Comment(_)) => {
                    // Skip any comments after the dash
                    stream.skip_newlines_and_comments()?;
                    // After skipping comments, check again for empty item
                    match stream.current() {
                        Some(Token::Newline) | None => {
                            if let Some((_, items)) = stack.last_mut() {
                                items.push(Node::None);
                            }
                            if let Some(Token::Newline) = stream.current() {
                                stream.next()?;
                            }
                        }
                        _ => {
                            // Parse the value after the comment
                            let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                            if let Some((_, items)) = stack.last_mut() {
                                items.push(value);
                            }
                        }
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
                        parent_indent,
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
                    } else if indent == current_indent
                        && matches!(stream.current(), Some(Token::Dash))
                    {
                        // Another dash at the same indent level - continue sequence
                        continue;
                    } else if indent >= current_indent {
                        use crate::parser::tokens::mapping::parse_mapping_with_tokens;
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
                    // Delegate trailing comma/closer consumption to centralized
                    // helper to keep flow punctuation behavior in one place while
                    // preserving existing semantics for block sequences.
                    flow_punctuation::consume_trailing_separators_and_closers_in_block_sequence(
                        stream,
                    )?;
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
                        use crate::parser::tokens::mapping::parse_mapping_with_tokens;
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

            // After parsing an item, skip whitespace/comments and check for another dash at the same indent.
            // Skip newlines and comments, but NOT indents (we need to check indent level)
            stream.skip_newlines_and_comments()?;

            // Check if there's another dash at the current indent level
            match stream.current() {
                Some(Token::Dash) => {
                    // Dash at same indent, continue parsing
                    continue;
                }
                Some(Token::Indent(level)) if *level < current_indent => {
                    // Dedent detected. If we have returned to or above the
                    // parent indentation, treat this as the natural end of
                    // the sequence; otherwise, the dedent is shallow (e.g.
                    // 4HVU: sequence under a key where a later item is less
                    // indented than its siblings but still more indented than
                    // the parent), which should be reported as an
                    // indentation error instead of silently ending the
                    // sequence and continuing as a separate structural block.
                    let dedent_level = *level;
                    if dedent_level <= parent_indent {
                        break;
                    } else {
                        return Err(crate::parser::document::error_builder::indentation_error(
                            stream.source_mut(),
                            "Invalid indentation for sequence item",
                        ));
                    }
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
    // Use NodeBuilder for final array node
    Ok(Node::Array(items))
}
