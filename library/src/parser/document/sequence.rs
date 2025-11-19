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
/// * `directives` - Directive context for tag resolution
///
/// # Returns
///
/// Result containing an Array Node or an error string
pub(crate) fn parse_sequence(
    source: &mut dyn ISource,
    indent_level: usize,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
    let mut items = Vec::new();
    let mut loop_iterations = 0;
    const MAX_LOOP_ITERATIONS: usize = 100_000;
    const MAX_ITEMS: usize = 50_000; // More reasonable limit on actual items

    while let Some(c) = source.current() {
        // Prevent infinite loop - count loop iterations for safety
        loop_iterations += 1;
        if loop_iterations >= MAX_LOOP_ITERATIONS {
            return Err(
                "Sequence parsing exceeded maximum loop iterations - possible infinite loop".to_string(),
            );
        }
        
        // Also check reasonable item count limit
        if items.len() >= MAX_ITEMS {
            return Err(
                format!("Sequence has too many items ({}+) - possible infinite loop", MAX_ITEMS)
            );
        }
        
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
                                directives,
                            )?);
                            continue;
                        }
                        CHAR_LBRACKET | CHAR_LBRACE => {
                            items.push(parse_value(source, directives)?);
                            continue;
                        }
                        _ => {
                            if crate::parser::document::helpers::peek_ahead_for_mapping_key(source)
                            {
                                let nested_indent = source.get_current_indent_level();
                                items.push(crate::parser::document::parse_document_contents(
                                    source,
                                    nested_indent,
                                    directives,
                                )?);
                            } else {
                                items.push(parse_value(source, directives)?);
                            }
                        }
                    }
                }
            }
            CHAR_NEWLINE | CHAR_CARRIAGE_RETURN => {
                source.next();
                skip_whitespace(source);
            }
            c if c.is_whitespace() => {
                skip_whitespace(source);
            }
            _ => {
                // Unexpected character - check if we should break or error
                if source.get_current_indent_level() <= indent_level {
                    break;
                }
                return Err(format!(
                    "Expected sequence item starting with CHAR_DASH, got '{}' at indent {}",
                    source.current().unwrap_or('\0'),
                    source.get_current_indent_level()
                ));
            }
        }
    }
    Ok(Node::Array(items))
}
