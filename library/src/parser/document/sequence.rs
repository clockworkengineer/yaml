//! Module: parser/document/sequence.rs

use crate::constants::*;
use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::document::context::{CollectionType, ParsingContext};
use crate::parser::document::helpers::{
    parse_comment, peek_ahead_for_document_start_end, skip_whitespace, validate_indentation,
    validate_no_tab_indentation,
};
use crate::parser::document::loop_guards::{MAX_LOOP_ITERATIONS, MAX_SEQUENCE_ITEMS};
use crate::parser::document::value::parse_value;
use crate::{combined_loop_guard, loop_guard_init};

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
    loop_guard_init!(loop_counter);

    // Track the indentation of the first dash for validation
    let mut first_dash_indent: Option<usize> = None;

    // Create parsing context for validation
    let mut ctx = ParsingContext::new(indent_level);
    ctx.collection_type = CollectionType::BlockSequence;

    while let Some(c) = source.current() {
        // Prevent infinite loop and excessive memory usage
        combined_loop_guard!(
            loop_counter,
            items,
            MAX_LOOP_ITERATIONS,
            MAX_SEQUENCE_ITEMS,
            "Sequence"
        );

        // Context-aware tab validation at line start
        if source.get_current_indent_level() == 0 || c == '\n' || c == '\r' {
            ctx.mark_newline_consumed();
            validate_indentation(source, &ctx)?;
            ctx.mark_content_found();
        }

        let current_indent = source.get_current_indent_level();

        // Special handling for dashes: validate indentation consistency
        // before breaking due to dedentation
        if c == CHAR_DASH && !peek_ahead_for_document_start_end(source, c) {
            if let Some(first_indent) = first_dash_indent {
                // We've seen at least one dash - validate this one matches
                if current_indent != first_indent {
                    if current_indent > first_indent {
                        // Nested sequence - will be handled by recursion
                        break;
                    } else if current_indent < indent_level {
                        // Dedented dash below the sequence's base indent - break to return to parent
                        break;
                    } else {
                        // Dash at inconsistent indentation within this sequence's range
                        return Err(crate::parser::document::helpers::parse_error(
                            source,
                            &format!(
                                "Inconsistent indentation in sequence: expected {}, got {}",
                                first_indent, current_indent
                            ),
                        ));
                    }
                }
            }
        }

        if current_indent < indent_level {
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
                // Record first dash indentation if not set
                if first_dash_indent.is_none() {
                    first_dash_indent = Some(current_indent);
                }

                source.next();
                // Tabs are not allowed after '-' if followed by structure or newline
                // But allowed if followed by content (e.g., "-\t-1" is valid, "-\t-" is not)
                if source.current() == Some('\t') {
                    // Save state and check what follows the tab
                    let state = source.save_state();
                    source.next(); // Skip tab
                    let next_char = source.current();
                    source.restore_state(state);

                    // Disallow tab if followed by newline, or by '-' that's not part of content
                    if next_char == Some('\n') || next_char == Some('\r') {
                        return Err(crate::parser::document::helpers::parse_error(
                            source,
                            "Tabs cannot be used as separation after sequence indicator",
                        ));
                    }
                    // If followed by another dash, check if it's a structure indicator
                    if next_char == Some('-') {
                        let state2 = source.save_state();
                        source.next(); // Skip tab
                        source.next(); // Skip dash
                        let after_dash = source.current();
                        source.restore_state(state2);
                        // If dash followed by space/newline, it's structure
                        if after_dash == Some(' ')
                            || after_dash == Some('\n')
                            || after_dash == Some('\r')
                            || after_dash == Some('\t')
                        {
                            return Err(crate::parser::document::helpers::parse_error(
                                source,
                                "Tabs cannot be used as separation after sequence indicator",
                            ));
                        }
                    }
                }
                skip_whitespace(source);
                // Handle both Unix (\n) and Windows (\r\n) line endings
                if source.current() == Some(CHAR_CARRIAGE_RETURN) {
                    source.next();
                }
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
                validate_no_tab_indentation(source)?;
                skip_whitespace(source);
            }
            c if c.is_whitespace() => {
                skip_whitespace(source);
            }
            _ => {
                // Check indent before throwing error
                let current_indent = source.get_current_indent_level();
                if current_indent <= indent_level {
                    // At or before sequence level - end of sequence
                    break;
                }

                // More indented - likely continuation of previous item or new nested content
                // Don't try to parse here - this creates loops. Just break and let parent handle it.
                match source.current() {
                    Some(c) => {
                        // If at correct indent and a mapping key, parse mapping as sequence item
                        if current_indent > indent_level {
                            use crate::parser::document::mapping::parse_mapping;
                            let mapping = parse_mapping(source, current_indent, directives)?;
                            items.push(mapping);
                            continue;
                        } else {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }
    Ok(Node::Array(items))
}
