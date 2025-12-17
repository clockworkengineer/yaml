use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::explicit_key::parse_multiple_explicit_keys;
use crate::parser::document::helpers;
use crate::parser::document::inline::{parse_inline_mapping, parse_inline_sequence};
use crate::parser::document::mapping::parse_mapping;
use crate::parser::document::sequence::parse_sequence;
use crate::parser::document::value::parse_value;

/// Fast token-dispatch for block constructs to prefer tokenized paths.
fn token_dispatch(
    source: &mut dyn ISource,
    directives: &DirectiveContext,
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
                source.restore_state(st);
                let seq_indent = source.get_current_indent_level();
                let mut stream =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)
                        .ok()?;
                let result = crate::parser::document::tokens::sequence::parse_sequence_with_tokens(
                    &mut stream,
                    seq_indent,
                    directives,
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
    Ok(parse_multiple_explicit_keys(source, current_indent)?)
}

/// Handles single explicit key logic using TokenStream-based parsing.
fn handle_single_explicit_key(
    source: &mut dyn ISource,
    directives: &DirectiveContext,
) -> Result<Node, String> {
    source.next();
    if source.current() == Some('\t') {
        let mut stream = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
        return Err(helpers::parse_error_token(
            &stream,
            "Tabs cannot be used as separation after explicit key indicator",
        ));
    }
    let mut stream = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
    stream.skip_whitespace_and_comments()?;
    let mut key_node = crate::parser::document::tokens::value::parse_value_with_tokens(
        &mut stream,
        directives,
        0,
    )?;
    use crate::nodes::node::BlockStyle;
    use crate::parser::document::helpers::node_to_inline_string;
    match key_node {
        Node::Array(_) | Node::Mapping(_) => {
            let inline = node_to_inline_string(&key_node);
            key_node = Node::Str(
                inline,
                crate::nodes::node::QuoteType::Double,
                BlockStyle::None,
            );
        }
        Node::Str(s, _qt, style) => {
            let key_string = if matches!(style, BlockStyle::Literal) {
                format!("{}\n", s)
            } else {
                s
            };
            key_node = Node::Str(
                key_string,
                crate::nodes::node::QuoteType::Double,
                BlockStyle::None,
            );
        }
        other => {
            let inline = node_to_inline_string(&other);
            key_node = Node::Str(
                inline,
                crate::nodes::node::QuoteType::Double,
                BlockStyle::None,
            );
        }
    }
    // ...existing code for colon and value parsing...
    let st_colon = source.save_state();
    let mut found_colon = false;
    loop {
        {
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, false)?;
            stream.skip_whitespace_and_comments()?;
        }
        match source.current() {
            Some(':') => {
                source.next();
                found_colon = true;
                break;
            }
            Some('\n') => {
                source.next();
                continue;
            }
            Some(_) | None => break,
        }
    }
    if !found_colon {
        source.restore_state(st_colon);
        if source.current() == Some('\n') {
            source.next();
        }
        loop {
            {
                let mut stream =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)?;
                stream.skip_whitespace_and_comments()?;
            }
            if source.current() == Some(':') {
                break;
            }
            if source.current().is_none() {
                break;
            }
            crate::utils::skip_until_newline(source);
            if source.current().is_none() {
                break;
            }
        }
    }
    if source.current() == Some(':') {
        source.next();
    }
    {
        let mut stream = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
        stream.skip_whitespace_and_comments()?;
    }
    let mut value_node = match source.current() {
        Some('[') => {
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, false)?;
            parse_inline_sequence(&mut stream, directives)?
        }
        Some('{') => {
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, false)?;
            parse_inline_mapping(&mut stream, directives)?
        }
        Some('-') => {
            let nested_indent = source.get_current_indent_level();
            parse_sequence(source, nested_indent, directives)?
        }
        Some(_) => parse_value(source, directives)?,
        None => Node::None,
    };
    if matches!(value_node, Node::None) {
        let st_peek = source.save_state();
        crate::utils::skip_whitespace_and_comments(source);
        if source.current() == Some('-') {
            let nested_indent = source.get_current_indent_level();
            value_node = parse_sequence(source, nested_indent, directives)?;
        } else {
            source.restore_state(st_peek);
        }
    }
    let mut pairs: Vec<(Node, Node)> = Vec::new();
    pairs.push((key_node, value_node));
    Ok(Node::Mapping(pairs))
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
) -> Result<Node, String> {
    if let Some(result) = token_dispatch(source, directives) {
        return result;
    }
    match source.current() {
        Some(c) if c == '-' => {
            let seq_indent = source.get_current_indent_level();
            // Removed is_doc_start check: do not skip top-level sequence after document marker
            let mut stream =
                crate::parser::token_stream::TokenStream::new(source, directives, false)?;
            match stream.current() {
                Some(crate::parser::lexer::Token::Dash) => {
                    if seq_indent < indent_level {
                        return Err(format!(
                            "Sequence item at invalid indentation: expected >= {}, got {}",
                            indent_level, seq_indent
                        ));
                    }
                    let seq =
                        crate::parser::document::tokens::sequence::parse_sequence_with_tokens(
                            &mut stream,
                            seq_indent,
                            directives,
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
            stream.skip_whitespace_and_comments()?;
            parse_document_contents(source, indent_level, directives)
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
                    stream.skip_whitespace_and_comments()?;
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
                    stream.skip_whitespace_and_comments()?;
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
                    let mut stream = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
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
            Ok(Node::Mapping(pairs))
        }
        Some(c) if c == '?' => {
            let current_indent = source.get_current_indent_level();
            let state = source.save_state();
            source.next();
            crate::utils::skip_until_newline(source);
            if source.current() == Some('\n') {
                source.next();
            }
            crate::utils::skip_whitespace_and_comments(source);
            let has_multiple_explicit_keys = source.get_current_indent_level() == current_indent
                && source.current() == Some('?');
            source.restore_state(state);
            if has_multiple_explicit_keys {
                return handle_multiple_explicit_keys(source, current_indent);
            } else {
                return handle_single_explicit_key(source, directives);
            }
        }
        Some(c) if c.is_alphanumeric() => {
            if helpers::peek_ahead_for_mapping_key(source, directives) {
                Ok(parse_mapping(source, indent_level, directives)?)
            } else {
                // If a plain word is followed by a newline and greater indentation
                // without a colon, this is likely a missing colon error.
                let state = source.save_state();
                // Create token stream to align with lexer boundaries (no assignment needed)
                let mut ts =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)?;
                match ts.current() {
                    Some(crate::parser::lexer::Token::Plain(_)) => {}
                    _ => {
                        // Fallback to scalar consumption to normalize position
                        let _ = ts.consume_plain_scalar()?;
                    }
                };
                // After consuming the current plain scalar, check newline + indent
                // Note: ts has advanced; reflect in source by restoring then consuming characters
                source.restore_state(state);
                // Read until end of line
                crate::utils::skip_until_newline(source);
                // If newline present, move to next char
                if source.current() == Some('\n') {
                    source.next();
                }
                let next_indent = source.get_current_indent_level();
                let st_after_nl = source.save_state();
                // Skip horizontal spaces
                while matches!(source.current(), Some(' ')) {
                    source.next();
                }
                let next_char = source.current();
                source.restore_state(st_after_nl);

                if next_indent > indent_level {
                    // If next significant char doesn't start a valid nested construct,
                    // report a missing colon.
                    if !matches!(
                        next_char,
                        Some('-') | Some('?') | Some('[') | Some('{') | Some('#')
                    ) {
                        let mut stream = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
                        return Err(helpers::parse_error_token(&stream, "Mapping key without colon"));
                    }
                }

                // Otherwise, treat as plain scalar
                let mut stream =
                    crate::parser::token_stream::TokenStream::new(source, directives, false)?;
                let s = stream.consume_plain_scalar()?;
                Ok(Node::Str(
                    s,
                    crate::nodes::node::QuoteType::Unquoted,
                    crate::nodes::node::BlockStyle::None,
                ))
            }
        }
        Some(c) if c.is_whitespace() => {
            source.next();
            Ok(parse_document_contents(source, indent_level, directives)?)
        }
        Some('\0') => {
            source.next();
            Ok(parse_document_contents(source, indent_level, directives)?)
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
        Some(c) => {
            let mut stream = crate::parser::token_stream::TokenStream::new(source, directives, false)?;
            Err(helpers::parse_error_token(
                &stream,
                &format!(
                    "{}{}",
                    crate::error::messages::ERR_UNEXPECTED_CHAR_PREFIX,
                    c
                ),
            ))
        },
        None => Ok(Node::None),
    }
}
