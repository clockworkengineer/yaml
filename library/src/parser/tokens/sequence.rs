//! Sequence Token Parsing
//!
//! Implements token-based parsing for YAML sequences, eliminating infinite loops
//! with decorators and simplifying sequence parsing logic.
//!
//! Copyright (c) 2026 YAML Library Developers

use crate::nodes::node::Node;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::flow_punctuation;
use crate::parser::lexer::Token;
use crate::parser::token_stream::TokenStream;
use crate::parser::tokens::value::parse_value_with_tokens;

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
use crate::parser::utils::context::ParsingContext;

/// Push `item` onto the top frame of the sequence parser's node-stack.
///
/// Every sequence item collection in `parse_sequence_with_tokens` goes through
/// this single call site, replacing the 13 identical inline `if let` blocks.
#[inline]
fn seq_stack_push(stack: &mut Vec<(usize, Vec<Node>)>, item: Node) {
    if let Some((_, items)) = stack.last_mut() {
        items.push(item);
    }
}

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
        // Save source position before skipping newlines. The lexer's look-ahead
        // may pre-fetch Token::DocumentStart/End (--- / ...) which advances the
        // underlying source past the marker characters, making the marker invisible
        // to the outer parse loop's is_document_marker check. Restore source to
        // this position when we are about to return due to a document boundary.
        let pre_skip_state = stream.source_mut().save_state();
        // Skip comments and newlines between sequence items
        // DO NOT skip Indent tokens here - we need them for dedent detection
        stream.skip_newlines_and_comments()?;
        // Only restore for DocumentStart (---): the lexer consuming past '---'
        // makes it invisible to the outer parser. DocumentEnd (...) does NOT need
        // restoration — its existing handling was already correct.
        if matches!(stream.current(), Some(Token::DocumentStart)) {
            stream.source_mut().restore_state(pre_skip_state);
        }
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
                crate::parser::utils::context::CollectionType::BlockMapping
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
                    crate::parser::utils::helpers::validate_trailing_content_after_document_end(
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
                        return Err(crate::parser::utils::error_builder::indentation_error(
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
                                seq_stack_push(&mut stack, mapping);
                            }
                        }
                        _ => {
                            // No indented content after the newline - this IS an empty item
                            seq_stack_push(&mut stack, Node::None);
                            // Continue to next iteration after handling empty item
                            continue;
                        }
                    }
                }
                None => {
                    // EOF after dash - empty item
                    seq_stack_push(&mut stack, Node::None);
                    continue;
                }
                Some(Token::Comment(_)) => {
                    // Skip any comments after the dash
                    stream.skip_newlines_and_comments()?;
                    // After skipping comments, check again for empty item
                    match stream.current() {
                        Some(Token::Newline) | None => {
                            seq_stack_push(&mut stack, Node::None);
                            if let Some(Token::Newline) = stream.current() {
                                stream.next()?;
                            }
                        }
                        _ => {
                            // Parse the value after the comment
                            let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                            seq_stack_push(&mut stack, value);
                        }
                    }
                }
                Some(Token::Dash) => {
                    // Nested sequence starts immediately (- - case)
                    // Use current_indent + 1 as base to properly detect dedents back to current level.
                    // Pass current_indent (not the outer parent_indent) as the parent for the nested
                    // call so that the indentation guard correctly treats a dedent back to
                    // current_indent as the natural end of the inline nested sequence rather than an
                    // error (e.g. ` - - itemA\n   - itemB\n - nextItem` must not error when the
                    // nested sequence closes back to the outer item indent).
                    let nested_base = current_indent + 1;
                    let ctx_seq = ctx.child_block_context(
                        nested_base,
                        crate::parser::utils::context::CollectionType::BlockSequence,
                    );
                    let seq = parse_sequence_with_tokens(
                        stream,
                        nested_base,
                        current_indent,
                        directives,
                        &ctx_seq,
                        depth + 1,
                    )?;
                    seq_stack_push(&mut stack, seq);
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
                        seq_stack_push(&mut stack, mapping);
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
                    seq_stack_push(&mut stack, value);
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
                        seq_stack_push(&mut stack, mapping);
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
                        seq_stack_push(&mut stack, value);
                    }
                }
                Some(Token::DoubleQuoted(..)) | Some(Token::SingleQuoted(_)) => {
                    // JKF3 / D49Q: Detect implicit block mapping where a quoted scalar
                    // is used as a key (followed immediately by ':').  This mirrors the
                    // existing Plain-scalar handling above.  Routing through
                    // parse_mapping_with_tokens lets the existing multiline-key guard in
                    // parse_mapping_pair fire for cases such as:
                    //   - - "bar\nbar": x   (JKF3 — continuation at indent 0 → error)
                    stream.skip_newlines_and_comments()?;
                    let is_colon = if matches!(
                        stream.current(),
                        Some(Token::DoubleQuoted(..)) | Some(Token::SingleQuoted(_))
                    ) {
                        matches!(stream.peek()?, Some(Token::Colon))
                    } else {
                        false
                    };
                    if is_colon {
                        use crate::parser::tokens::mapping::parse_mapping_with_tokens;
                        let indent = current_indent;
                        let mapping =
                            parse_mapping_with_tokens(stream, indent, directives, depth + 1)?;
                        seq_stack_push(&mut stack, mapping);
                        stream.skip_newlines_and_comments()?;
                    } else {
                        let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                        seq_stack_push(&mut stack, value);
                    }
                }
                _ => {
                    let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                    seq_stack_push(&mut stack, value);
                }
            }

            // After parsing an item, skip whitespace/comments and check for another dash at the same indent.
            // Skip newlines and comments, but NOT indents (we need to check indent level)
            stream.skip_newlines_and_comments()?;

            // Check if there's another dash at the current indent level
            match stream.current() {
                Some(Token::Dash) => {
                    // A Dash here means parse_plain_scalar consumed the preceding
                    // Indent token while probing for a multiline continuation and
                    // left the Dash when it found the line wasn't a plain scalar.
                    // stream.line_indent() still reflects the indent level of that
                    // consumed Indent token, so we can validate alignment:
                    //   • indent < current_indent AND indent > parent_indent → the
                    //     dash is at an in-between level that is neither a sibling
                    //     item nor a natural dedent back to the parent.  This is the
                    //     4HVU case: error.
                    //   • indent <= parent_indent → dedent past the sequence base;
                    //     end the sequence cleanly.
                    //   • indent == current_indent → valid sibling; carry on.
                    //   • indent > current_indent → ZVH3: wrongly indented item;
                    //     nested sequences are handled during value parsing so a
                    //     deeper dash here is always an indentation error.
                    let dash_indent = stream.line_indent();
                    if dash_indent < current_indent {
                        if dash_indent <= parent_indent {
                            break;
                        } else {
                            return Err(crate::parser::utils::error_builder::indentation_error(
                                stream.source_mut(),
                                "Invalid indentation for sequence item",
                            ));
                        }
                    }
                    if dash_indent > current_indent {
                        // ZVH3: a deeper-indented dash appearing after a *mapping*
                        // item is a true indentation error — the mapping consumed
                        // the Indent token and returned when it saw the Dash, so the
                        // Dash cannot legitimately start a sibling or nested item.
                        //
                        // By contrast, when the previous item was a scalar, this
                        // path is reached because the plain-scalar prober consumed
                        // the Indent token while probing for multiline continuation
                        // and backed off at the Dash (AB8U pattern). In that case
                        // the Dash may validly continue or extend the sequence, so
                        // we must NOT error.
                        let last_was_mapping = stack
                            .last()
                            .and_then(|(_, items)| items.last())
                            .map_or(false, |n| matches!(n, crate::nodes::node::Node::Mapping(_)));
                        if last_was_mapping {
                            return Err(crate::parser::utils::error_builder::indentation_error(
                                stream.source_mut(),
                                "Invalid indentation for sequence item",
                            ));
                        }
                    }
                    // Dash at same indent — valid sibling item.
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
                        return Err(crate::parser::utils::error_builder::indentation_error(
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
                    } else if matches!(stream.current(), Some(Token::Anchor(_))) {
                        // GT5M: an anchor on its own line at the sequence indentation
                        // level (without a preceding '-') is always a syntax error —
                        // anchors must annotate a node and every block sequence item
                        // must begin with '-'. The 'current_indent > parent_indent'
                        // guard used for plain scalars does NOT apply here because an
                        // anchor can never legitimately stand alone at this level.
                        return Err(crate::parser::utils::error_builder::syntax_error(
                            stream.source_mut(),
                            "Anchor on its own line at block sequence level (missing '-')",
                        ));
                    } else if matches!(
                        stream.current(),
                        Some(Token::Plain(_))
                            | Some(Token::SingleQuoted(_))
                            | Some(Token::DoubleQuoted(..))
                            | Some(Token::Tag(_))
                    ) && current_indent > parent_indent
                    {
                        // 6S55: A scalar (or other non-Dash node-starter) appearing at
                        // the same indentation level as the sequence items is always
                        // invalid in a block sequence — every item must begin with '-'.
                        // Only reject when we're inside a genuinely nested sequence
                        // (current_indent > parent_indent); at the top level both are
                        // typically 0 so the token legitimately ends the sequence.
                        return Err(crate::parser::utils::error_builder::indentation_error(
                            stream.source_mut(),
                            "Invalid scalar at sequence level (expected '-')",
                        ));
                    } else {
                        break;
                    }
                }
                _ => {
                    // GT5M (II): When the plain-scalar prober consumed the Indent token
                    // and left an Anchor as the next token, an anchor at the sequence's
                    // indentation level (without '-') is always a syntax error.
                    if matches!(stream.current(), Some(Token::Anchor(_)))
                        && stream.line_indent() == current_indent
                    {
                        return Err(crate::parser::utils::error_builder::syntax_error(
                            stream.source_mut(),
                            "Anchor on its own line at block sequence level (missing '-')",
                        ));
                    }
                    // 6S55 (II): The plain-scalar prober may consume the
                    // Indent(current_indent) token and leave a Plain/Quoted
                    // scalar as the next current token.  If that scalar sits
                    // at the same column as the sequence items, it is not a
                    // valid block-sequence entry (missing '-') and must be
                    // rejected in a nested sequence context.
                    if matches!(
                        stream.current(),
                        Some(Token::Plain(_))
                            | Some(Token::SingleQuoted(_))
                            | Some(Token::DoubleQuoted(..))
                            | Some(Token::Tag(_))
                    ) && current_indent > parent_indent
                        && stream.line_indent() == current_indent
                    {
                        return Err(crate::parser::utils::error_builder::indentation_error(
                            stream.source_mut(),
                            "Invalid scalar at sequence level (expected '-')",
                        ));
                    }
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

/// Convenience wrapper: build a block-sequence context at `level` and call
/// `parse_sequence_with_tokens`.  Eliminates the five-line boilerplate that
/// appears at every Dash-dispatch site in the mapping and value parsers.
pub fn parse_block_sequence_at(
    stream: &mut TokenStream,
    level: usize,
    parent_indent: usize,
    directives: &DirectiveContext,
    depth: usize,
) -> crate::parser::ParseResult<Node> {
    let ctx = ParsingContext::new(level).child_block_context(
        level,
        crate::parser::utils::context::CollectionType::BlockSequence,
    );
    parse_sequence_with_tokens(stream, level, parent_indent, directives, &ctx, depth)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_zvh3_wrong_indented_sequence_item_should_error() {
        // ZVH3: `- key: value\n - item1\n`
        // The second dash is at indent 1 while the sequence is at indent 0 — invalid.
        let yaml = "- key: value\n - item1\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_err(),
            "ZVH3 should fail: '- key: value\\n - item1' has wrong indented sequence item, got: {:?}",
            result
        );
    }

    #[test]
    fn test_3alj_nested_sequences_should_succeed() {
        let yaml = "- - s1_i1\n  - s1_i2\n- s2\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_ok(),
            "3ALJ should succeed, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_ab8u_multiline_plain_scalar_should_succeed() {
        let yaml = "- single multiline\n - sequence entry\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_ok(),
            "AB8U should succeed, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_6bct_nested_sequence_in_sequence_should_succeed() {
        let yaml = "- foo:   bar\n- - baz\n  -     baz\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_ok(),
            "6BCT should succeed, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_6s55_scalar_at_sequence_level_should_error() {
        // 6S55: plain scalar at the same indent as sequence items (no '-') is invalid
        let yaml = "key:\n - bar\n - baz\n invalid\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_err(),
            "6S55 should fail: scalar at sequence level without '-', got: {:?}",
            result
        );
    }

    #[test]
    fn test_gt5m_anchor_on_own_line_at_seq_level_should_error() {
        // GT5M: anchor on its own line at the sequence indentation level (without '-') is invalid
        let yaml = "- item1\n&node\n- item2\n";
        let config = crate::parser::config::ParserConfig::strict();
        let result = crate::parse_with_config(yaml, config);
        assert!(
            result.is_err(),
            "GT5M should fail: anchor on its own line at block sequence level without '-', got: {:?}",
            result
        );
    }
}
