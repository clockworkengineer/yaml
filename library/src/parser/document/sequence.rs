//! Module: parser/document/sequence.rs

use crate::constants::*;
use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::document::helpers::{
    parse_comment, peek_ahead_for_document_start_end, skip_whitespace,
};
use crate::parser::document::value::parse_value;

/// Parses a YAML sequence (array) with the specified indentation level.
///
/// Processes sequence items marked with '-' at the beginning of lines,
/// handling nested sequences, comments, and document boundaries.
/// Maintains proper indentation tracking for nested structures.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `indent_level` - The expected indentation level for sequence items
///
/// # Returns
///
/// Result containing an Array Node or an error string
pub(crate) fn parse_sequence(
    source: &mut dyn ISource,
    indent_level: usize,
) -> Result<Node, String> {
    let mut items = Vec::new();
    while let Some(c) = source.current() {
        if source.get_current_indent_level() < indent_level {
            break;
        }

        match c {
            CHAR_HASH => {
                parse_comment(source);
                if source.current() == Some(CHAR_NEWLINE) {
                    source.next();
                }
                skip_whitespace(source);
                continue;
            }
            CHAR_DASH | CHAR_DOT if peek_ahead_for_document_start_end(source, c) => {
                break;
            }
            CHAR_DASH => {
                source.next();
                skip_whitespace(source);
                if source.current() == Some(CHAR_NEWLINE) {
                    source.next();
                    skip_whitespace(source);
                }

                if let Some(next_c) = source.current() {
                    match next_c {
                        CHAR_DASH => {
                            let nested_indent = source.get_current_indent_level();
                            items.push(crate::parser::document::parse_document_contents(
                                source,
                                nested_indent,
                            )?);
                            continue;
                        }
                        CHAR_LBRACKET | CHAR_LBRACE => {
                            items.push(parse_value(source)?);
                            continue;
                        }
                        _ => {
                            if crate::parser::document::helpers::peek_ahead_for_mapping_key(source)
                            {
                                let nested_indent = source.get_current_indent_level();
                                items.push(crate::parser::document::parse_document_contents(
                                    source,
                                    nested_indent,
                                )?);
                                continue;
                            } else {
                                items.push(parse_value(source)?);
                            }
                        }
                    }
                }
            }
            _ if !c.is_whitespace() => {
                return Err(format!(
                    "Expected sequence item starting with CHAR_DASH, got '{c}'"
                ));
            }
            _ => (),
        }

        crate::utils::skip_until_newline(source);
        skip_whitespace(source);
    }
    Ok(Node::Array(items))
}
