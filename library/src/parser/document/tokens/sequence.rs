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
    let mut items = Vec::new();

    // Skip initial whitespace/newlines but track where we start
    stream.skip_whitespace()?;

    loop {
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
            if matches!(
                stream.current(),
                Some(Token::FlowMappingEnd) | Some(Token::FlowSequenceEnd)
            ) {
                stream.next()?;
            }
            stream.skip_whitespace_and_comments()?;
        }
        // End sequence if dedent or document marker, but only if not in explicit key or flow context
        if !ctx.in_flow
            && !matches!(
                ctx.collection_type,
                crate::parser::document::context::CollectionType::BlockMapping
            )
        {
            match stream.current() {
                Some(Token::DocumentStart) | Some(Token::DocumentEnd) => {
                    if items.is_empty() {
                        return Ok(Node::None);
                    } else {
                        break;
                    }
                }
                Some(Token::Indent(level)) if *level < base_indent => {
                    break;
                }
                _ => {}
            }
        }

        // Only parse a sequence item if the current token is a dash at the current indent
        let is_dash = matches!(stream.current(), Some(Token::Dash));
        if is_dash {
            stream.next()?;
            stream.skip_whitespace_and_comments()?;
            match stream.current() {
                Some(Token::Newline) | None => {
                    items.push(Node::None);
                    if let Some(Token::Newline) = stream.current() {
                        stream.next()?;
                    }
                }
                Some(Token::Dash) => {
                    let ctx_seq = ctx.child_block_context(
                        base_indent,
                        crate::parser::document::context::CollectionType::BlockSequence,
                    );
                    let seq = parse_sequence_with_tokens(
                        stream,
                        base_indent,
                        directives,
                        &ctx_seq,
                        depth + 1,
                    )?;
                    items.push(seq);
                }
                Some(Token::Indent(level)) => {
                    let indent = *level;
                    stream.next()?;
                    if indent > base_indent && matches!(stream.current(), Some(Token::Dash)) {
                        let ctx_seq = ctx.child_block_context(
                            indent,
                            crate::parser::document::context::CollectionType::BlockSequence,
                        );
                        let seq = parse_sequence_with_tokens(
                            stream,
                            indent,
                            directives,
                            &ctx_seq,
                            depth + 1,
                        )?;
                        items.push(seq);
                    } else if indent >= base_indent {
                        use crate::parser::document::tokens::mapping::parse_mapping_with_tokens;
                        let mapping =
                            parse_mapping_with_tokens(stream, indent, directives, depth + 1)?;
                        items.push(mapping);
                    } else {
                        break;
                    }
                    while matches!(
                        stream.current(),
                        Some(Token::Newline) | Some(Token::Comment(_))
                    ) {
                        stream.next()?;
                    }
                }
                Some(Token::FlowSequenceStart) | Some(Token::FlowMappingStart) => {
                    let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                    items.push(value);
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
                    while matches!(stream.current(), Some(Token::Newline)) {
                        stream.next()?;
                    }
                }
                Some(Token::Plain(_)) => {
                    stream.skip_whitespace_and_comments()?;
                    let mut is_colon = false;
                    if let Some(Token::Plain(_)) = stream.current() {
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
                        stream.skip_whitespace_and_comments()?;
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
                        while matches!(
                            stream.current(),
                            Some(Token::Newline) | Some(Token::Comment(_))
                        ) {
                            stream.next()?;
                        }
                    } else {
                        let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                        items.push(value);
                    }
                }
                _ => {
                    let value = parse_value_with_tokens(stream, directives, depth + 1)?;
                    items.push(value);
                }
            }

            // After parsing an item, skip whitespace/comments and check for another dash at the same indent
            stream.skip_whitespace_and_comments()?;
            if matches!(stream.current(), Some(Token::Dash)) {
                continue;
            } else {
                break;
            }
        } else {
            // If not a dash, skip to next token or break if at end
            break;
        }
    }
    Ok(Node::Array(items))
}
