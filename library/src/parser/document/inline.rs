//! Module: parser/document/inline.rs

use crate::constants::*;
use crate::error::messages::*;
use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::document::helpers::{parse_error, parse_quoted_scalar, skip_whitespace};
use crate::parser::document::scalar::parse_scalar;
use crate::utils::*;

/// Parses an inline YAML mapping enclosed in curly braces {}.
///
/// Handles comma-separated key-value pairs within braces, including
/// nested inline collections, quoted strings, and whitespace handling.
/// Supports empty mappings and nested structures.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
///
/// # Returns
///
/// Result containing a Mapping Node or an error string
pub(crate) fn parse_inline_mapping(source: &mut dyn ISource) -> Result<Node, String> {
    let mut pairs: Vec<(Node, Node)> = Vec::new();
    source.next();
    skip_whitespace(source);

    if source.current() == Some(CHAR_RBRACE) {
        source.next();
        return Ok(Node::Mapping(pairs));
    }

    loop {
        let key_node = {
            let raw = match source.current() {
                Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => parse_quoted_scalar(source)?,
                _ => collect_until(source, |c| c == CHAR_COLON || c == CHAR_RBRACE),
            };
            if source.current() != Some(CHAR_COLON) {
                return Err(parse_error(source, ERR_EXPECT_COLON_INLINE_MAPPING));
            }
            source.next();
            let trimmed = raw.trim();
            parse_scalar(trimmed)
        };

        skip_whitespace(source);

        let value_node = match source.current() {
            Some(CHAR_LBRACE) => parse_inline_mapping(source)?,
            Some(CHAR_LBRACKET) => parse_inline_sequence(source)?,
            Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                let raw = parse_quoted_scalar(source)?;
                parse_scalar(raw.trim())
            }
            Some(_) => {
                let val = collect_until(source, |c| {
                    c == CHAR_COMMA || c == CHAR_RBRACE || c == CHAR_HASH
                });
                parse_scalar(val.trim())
            }
            None => return Err(parse_error(source, ERR_EOF_INLINE_MAPPING)),
        };

        pairs.push((key_node, value_node));

        skip_whitespace_and_comments(source);

        match source.current() {
            Some(CHAR_COMMA) => {
                source.next();
                skip_whitespace(source);
                continue;
            }
            Some(CHAR_RBRACE) => {
                source.next();
                break;
            }
            Some(c) => {
                return Err(parse_error(
                    source,
                    &format!("{ERR_UNEXPECTED_CHAR_INLINE_MAPPING_PREFIX}{c}"),
                ));
            }
            None => return Err(parse_error(source, ERR_EOF_INLINE_MAPPING)),
        }
    }

    Ok(Node::Mapping(pairs))
}

/// Parses an inline YAML sequence enclosed in square brackets [].
///
/// Handles comma-separated values within brackets, including nested
/// inline collections, quoted strings, and proper whitespace handling.
/// Supports empty sequences and nested structures.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
///
/// # Returns
///
/// Result containing an Array Node or an error string
pub(crate) fn parse_inline_sequence(source: &mut dyn ISource) -> Result<Node, String> {
    let mut items: Vec<Node> = Vec::new();
    source.next();
    skip_whitespace(source);

    if source.current() == Some(CHAR_RBRACKET) {
        source.next();
        return Ok(Node::Array(items));
    }

    loop {
        match source.current() {
            Some(CHAR_LBRACKET) => {
                let nested = parse_inline_sequence(source)?;
                items.push(nested);
            }
            Some(CHAR_LBRACE) => {
                let nested_map = parse_inline_mapping(source)?;
                items.push(nested_map);
            }
            Some(_) => {
                let node = match source.current() {
                    Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                        let raw = parse_quoted_scalar(source)?;
                        parse_scalar(raw.trim())
                    }
                    _ => {
                        let val = collect_until(source, |c| {
                            c == CHAR_COMMA || c == CHAR_RBRACKET || c == CHAR_HASH
                        });
                        let trimmed = val.trim();
                        if trimmed.is_empty() {
                            Node::None
                        } else {
                            parse_scalar(trimmed)
                        }
                    }
                };
                if !matches!(node, Node::None) {
                    items.push(node);
                }
            }
            None => return Err(parse_error(source, ERR_EOF_INLINE_SEQUENCE)),
        }

        skip_whitespace_and_comments(source);

        match source.current() {
            Some(CHAR_COMMA) => {
                source.next();
                skip_whitespace(source);
                continue;
            }
            Some(CHAR_RBRACKET) => {
                source.next();
                break;
            }
            Some(c) => {
                return Err(parse_error(
                    source,
                    &format!("{ERR_UNEXPECTED_CHAR_INLINE_SEQUENCE_PREFIX}{c}"),
                ));
            }
            None => return Err(parse_error(source, ERR_EOF_INLINE_SEQUENCE)),
        }
    }

    Ok(Node::Array(items))
}
