use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::context::{CollectionType, ParsingContext};
use crate::parser::document::explicit_key::parse_multiple_explicit_keys;
use crate::parser::document::helpers;
use crate::parser::document::mapping::parse_mapping;
use crate::parser::document::value::parse_value;

/// Fast token-dispatch for block constructs to prefer tokenized paths.
fn token_dispatch(
    source: &mut dyn ISource,
    directives: &DirectiveContext,
    ctx: &ParsingContext,
) -> Option<Result<Node, String>> {
    let st = source.save_state();
    if let Ok(ts) = crate::parser::token_stream::TokenStream::new(source, directives, false) {
        match ts.current() {
            Some(crate::parser::lexer::Token::Indent(lvl)) => {
                let level_val = *lvl;
                source.restore_state(st);
                let mut stream =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)
                        .ok()?;
                let result = crate::parser::document::tokens::mapping::parse_mapping_with_tokens(
                    &mut stream,
                    level_val,
                    directives,
                    0,
                );
                return Some(result);
            }
            Some(crate::parser::lexer::Token::Dash) => {
                // Only treat dash as sequence start if not in flow context or explicit key context
                if ctx.in_flow || matches!(ctx.collection_type, CollectionType::BlockMapping) {
                    source.restore_state(st);
                    return None;
                }
                source.restore_state(st);
                let seq_indent = source.get_current_indent_level();
                let mut stream =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)
                        .ok()?;
                let ctx_seq = ctx.child_block_context(seq_indent, CollectionType::BlockSequence);
                let result = crate::parser::document::tokens::sequence::parse_sequence_with_tokens(
                    &mut stream,
                    seq_indent,
                    directives,
                    &ctx_seq,
                    0,
                );
                return Some(result);
            }
            _ => {
                source.restore_state(st);
            }
        }
    } else {
        source.restore_state(st);
    }
    None
}

/// Checks if the current position is at a document end marker (...).
fn is_doc_end(source: &mut dyn ISource, directives: &DirectiveContext) -> Result<bool, String> {
    let st = source.save_state();
    let ts = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
    let res = matches!(ts.current(), Some(crate::parser::lexer::Token::DocumentEnd));
    source.restore_state(st);
    Ok(res)
}

/// Handles multiple explicit keys at the same indentation level.
fn handle_multiple_explicit_keys(
    source: &mut dyn ISource,
    current_indent: usize,
) -> Result<Node, String> {
    // Now collects (key, value) pairs for all explicit keys at this indent
    Ok(parse_multiple_explicit_keys(source, current_indent)?)
}


/// Parses the contents of a YAML document based on the current character and context.
///
/// Determines the appropriate parsing strategy based on the current character:
/// sequences (-), comments (#), inline mappings ({}), inline sequences ([]),
/// explicit mapping keys (?), block scalars (| or >), or regular mappings.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `indent_level` - The indentation level for proper nesting
/// * `directives` - Directive context for tag resolution and version-specific parsing
///
/// # Returns
///
/// Result containing a Node or an error string
pub fn parse_document_contents(
    source: &mut dyn ISource,
    indent_level: usize,
    directives: &DirectiveContext,
    ctx: &ParsingContext,
) -> Result<Node, String> {
    // Normalize position to the next significant token
    crate::utils::skip_whitespace_and_comments(source);
    // Handle explicit keys first so token-dispatch doesn't overshadow '?'
    if matches!(source.current(), Some('?')) {
        // Always collect all consecutive explicit keys at this indent into a single mapping node
        let current_indent = source.get_current_indent_level();
        return handle_multiple_explicit_keys(source, current_indent);
    }
    // (debug removed)
    if let Some(result) = token_dispatch(source, directives, ctx) {
        return result;
    }
    match source.current() {
        Some(c) if c == '-' => {
            // Only treat dash as sequence start if not in flow context or explicit key context
            if ctx.in_flow || matches!(ctx.collection_type, CollectionType::BlockMapping) {
                return Ok(parse_value(source, directives)?);
            }
            let seq_indent = source.get_current_indent_level();
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, false)?;
            match stream.current() {
                // If this is actually a document start marker (---), do not parse as a sequence.
                // Leave the marker for the main loop to handle by returning Node::None without consuming.
                Some(crate::parser::lexer::Token::DocumentStart) => {
                    return Ok(Node::None);
                }
                Some(crate::parser::lexer::Token::Dash) => {
                    // seq_indent already captured above
                    if seq_indent < indent_level {
                        return Err(format!(
                            "Sequence item at invalid indentation: expected >= {}, got {}",
                            indent_level, seq_indent
                        ));
                    }
                    let ctx_seq =
                        ctx.child_block_context(seq_indent, CollectionType::BlockSequence);
                    let seq =
                        crate::parser::document::tokens::sequence::parse_sequence_with_tokens(
                            &mut stream,
                            seq_indent,
                            directives,
                            &ctx_seq,
                            0,
                        )?;
                    if let Node::None = seq {
                        return Ok(Node::None);
                    }
                    Ok(seq)
                }
                _ => Ok(parse_value(source, directives)?),
            }
        }
        Some(c) if c == '.' => {
            let map_indent = source.get_current_indent_level();
            if is_doc_end(source, directives)? {
                // Always break to main document loop, even if deeply nested
                return Ok(Node::None);
            }
            if map_indent < indent_level {
                return Err(format!(
                    "Mapping key at invalid indentation: expected >= {}, got {}",
                    indent_level, map_indent
                ));
            }
            if helpers::peek_ahead_for_mapping_key(source, directives) {
                Ok(parse_mapping(source, map_indent, directives)?)
            } else {
                Ok(parse_value(source, directives)?)
            }
        }
        Some(c) if c == '#' => {
            // Use token stream to skip comments and whitespace uniformly
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, false)?;
            stream.skip_trivia()?;
            parse_document_contents(source, indent_level, directives, ctx)
        }
        Some(c) if c == '{' => {
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, true)?;
            Ok(
                crate::parser::document::inline_tokens::parse_inline_mapping_with_tokens(
                    &mut stream,
                    directives,
                    0,
                    false,
                )?,
            )
        }
        Some(c) if c == '[' => {
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, true)?;
            Ok(
                crate::parser::document::inline_tokens::parse_inline_sequence_with_tokens(
                    &mut stream,
                    directives,
                    0,
                )?,
            )
        }
        // Support tagged values or tagged keys using TokenStream
        Some(c) if c == '!' => {
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, false)?;
            // If a tag token appears and is followed by a colon at same level, treat as mapping key
            match stream.current() {
                Some(crate::parser::lexer::Token::Tag(_)) => {
                    stream.next()?;
                    stream.skip_trivia()?;
                    let is_mapping_key =
                        matches!(stream.current(), Some(crate::parser::lexer::Token::Colon));
                    if is_mapping_key {
                        Ok(parse_mapping(source, indent_level, directives)?)
                    } else {
                        Ok(
                            crate::parser::document::tokens::value::parse_value_with_tokens(
                                &mut stream,
                                directives,
                                0,
                            )?,
                        )
                    }
                }
                _ => Ok(
                    crate::parser::document::tokens::value::parse_value_with_tokens(
                        &mut stream,
                        directives,
                        0,
                    )?,
                ),
            }
        }
        // Support anchors using TokenStream
        Some(c) if c == '&' => {
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, false)?;
            match stream.current() {
                Some(crate::parser::lexer::Token::Anchor(_)) => {
                    stream.next()?;
                    stream.skip_trivia()?;
                    let is_mapping_key =
                        matches!(stream.current(), Some(crate::parser::lexer::Token::Colon));
                    if is_mapping_key {
                        Ok(parse_mapping(source, indent_level, directives)?)
                    } else {
                        Ok(
                            crate::parser::document::tokens::value::parse_value_with_tokens(
                                &mut stream,
                                directives,
                                0,
                            )?,
                        )
                    }
                }
                _ => Ok(
                    crate::parser::document::tokens::value::parse_value_with_tokens(
                        &mut stream,
                        directives,
                        0,
                    )?,
                ),
            }
        }
        // Support aliases at document level (e.g. "*anchor")
        Some(c) if c == '*' => Ok(crate::parser::document::value::parse_value(
            source, directives,
        )?),
        // Handle explicit value indicator (: value) with missing/null key using tokens
        Some(c) if c == ':' => {
            let current_indent = source.get_current_indent_level();
            let mut pairs: Vec<(Node, Node)> = Vec::new();
            loop {
                if source.get_current_indent_level() != current_indent
                    || source.current() != Some(':')
                {
                    break;
                }
                source.next();
                if source.current() == Some('\t') {
                    let stream =
                        crate::parser::token_stream::TokenStream::new(source, directives, false)?;
                    return Err(helpers::parse_error_token(
                        &stream,
                        "Tabs cannot be used as separation after explicit value indicator",
                    ));
                }
                crate::utils::skip_whitespace_and_comments(source);

                let mut stream =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)?;
                let value_node = match stream.current() {
                    Some(crate::parser::lexer::Token::Newline) | None => Node::None,
                    _ => crate::parser::document::tokens::value::parse_value_with_tokens(
                        &mut stream,
                        directives,
                        0,
                    )?,
                };

                pairs.push((Node::None, value_node));

                crate::utils::skip_until_newline(source);
                if source.current() == Some('\n') {
                    source.next();
                }
                crate::utils::skip_whitespace_and_comments(source);
            }
            Ok(crate::parser::document::node_utils::make_mapping_node(pairs))
        }
        Some(c) if c == '?' => unreachable!(),
        Some(c) if c.is_alphanumeric() => {
            if helpers::peek_ahead_for_mapping_key(source, directives) {
                // Prefer token-based mapping parsing for reliability
                let base_indent = source.get_current_indent_level();
                let mut stream =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)?;
                Ok(
                    crate::parser::document::tokens::mapping::parse_mapping_with_tokens(
                        &mut stream,
                        base_indent,
                        directives,
                        0,
                    )?,
                )
            } else {
                // Fallback: quick token check for Plain followed by Colon on same line
                let st_map_check = source.save_state();
                if let Ok(mut ts) =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)
                {
                    if matches!(ts.current(), Some(crate::parser::lexer::Token::Plain(_))) {
                        let _ = ts.next();
                        if matches!(ts.current(), Some(crate::parser::lexer::Token::Colon)) {
                            source.restore_state(st_map_check);
                            return Ok(parse_mapping(source, indent_level, directives)?);
                        }
                    }
                }
                source.restore_state(st_map_check);
                // If a plain word is followed by a newline and greater indentation
                // without a colon, this is likely a missing colon error.
                let _state = source.save_state();
                // Create token stream to align with lexer boundaries (no assignment needed)
                let seq_indent = source.get_current_indent_level();
                let mut stream =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)?;
                match stream.current() {
                    // If this is actually a document start marker (---), do not parse as a sequence.
                    // Leave the marker for the main loop to handle by returning Node::None without consuming.
                    Some(crate::parser::lexer::Token::DocumentStart) => {
                        return Ok(Node::None);
                    }
                    _ => {}
                }
                let ctx_seq = ctx.child_block_context(seq_indent, CollectionType::BlockSequence);
                let node = crate::parser::document::tokens::sequence::parse_sequence_with_tokens(
                    &mut stream,
                    seq_indent,
                    directives,
                    &ctx_seq,
                    0,
                )?;
                return Ok(node);
            }
        }
        Some(c) if c.is_whitespace() => {
            source.next();
            Ok(parse_document_contents(
                source,
                indent_level,
                directives,
                ctx,
            )?)
        }
        Some('\0') => {
            source.next();
            Ok(parse_document_contents(
                source,
                indent_level,
                directives,
                ctx,
            )?)
        }
        Some(c) if matches!(c, '<' | '>' | '"' | '\'' | '|') => {
            if matches!(source.current(), Some('"') | Some('\''))
                && helpers::peek_ahead_for_mapping_key(source, directives)
            {
                Ok(parse_mapping(source, indent_level, directives)?)
            } else {
                Ok(parse_value(source, directives)?)
            }
        }
        Some('%') => {
            // A directive start encountered within content parsing: signal no node here.
            // The main document loop will handle breaking at directives.
            Ok(Node::None)
        }
        Some(c) => {
            let stream = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
            Err(helpers::parse_error_token(
                &stream,
                &format!(
                    "{}{}",
                    crate::error::messages::ERR_UNEXPECTED_CHAR_PREFIX,
                    c
                ),
            ))
        }
        None => Ok(Node::None),
    }
}
