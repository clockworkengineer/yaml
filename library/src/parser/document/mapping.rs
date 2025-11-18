//! Module: parser/document/mapping.rs

use crate::constants::*;
use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::nodes::node::{BlockStyle, QuoteType};
use crate::parser::document::helpers::{
    parse_comment, parse_error, parse_mapping_key,
};
use crate::parser::document::value::parse_value;

/// Parses a YAML mapping (dictionary) with the specified indentation level.
///
/// Processes key-value pairs, handling complex keys, nested mappings,
/// comments, and proper indentation. Determines appropriate quoting
/// for keys and values based on content safety rules.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `indent_level` - The expected indentation level for mapping entries
/// * `directives` - Directive context for tag resolution
///
/// # Returns
///
/// Result containing a Mapping Node or an error string
pub(crate) fn parse_mapping(
    source: &mut dyn ISource,
    indent_level: usize,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
    /// Checks if a string value can be safely represented as plain (unquoted) YAML.
    ///
    /// Returns false if the string contains characters that require quoting
    /// or has leading/trailing spaces that would be lost in plain format.
    fn is_plain_safe_value(s: &str) -> bool {
        if s.is_empty() {
            return true;
        }
        if s.starts_with(' ') || s.ends_with(' ') {
            return false;
        }
        if s.contains(['\n', '\r']) {
            return false;
        }
        let disallowed = [
            '#', '[', ']', '{', '}', '&', '*', '!', '|', '>', '"', '`', '%', '@', '\\',
        ];
        if s.chars().any(|ch| disallowed.contains(&ch)) {
            return false;
        }
        true
    }
    /// Checks if a string key can be safely represented as plain (unquoted) YAML.
    ///
    /// Similar to is_plain_safe_value but additionally excludes colons which
    /// have special meaning in YAML key-value syntax.
    fn is_plain_safe_key(s: &str) -> bool {
        is_plain_safe_value(s) && !s.contains(':')
    }

    let mut pairs: Vec<(Node, Node)> = Vec::new();
    let mut last_was_nested: bool;
    while let Some(c) = source.current() {
        last_was_nested = false;
        match c {
            CHAR_DASH | CHAR_DOT
                if crate::parser::document::helpers::peek_ahead_for_document_start_end(
                    source, c,
                ) =>
            {
                break;
            }
            CHAR_HASH => {
                parse_comment(source);
            }
            c if c.is_alphanumeric() || c == CHAR_SINGLE_QUOTE || c == CHAR_DOUBLE_QUOTE => {
                if source.get_current_indent_level() < indent_level {
                    break;
                }
                let (mut key_node, newline) = parse_mapping_key(source, directives)?;
                if let Node::Str(ref mut s, ref mut qt, ref mut _style) = key_node {
                    if matches!(*qt, QuoteType::Single | QuoteType::Double) && is_plain_safe_key(s)
                    {
                        *qt = QuoteType::Unquoted;
                    }
                }

                // Check for sequence on same line as mapping key (error case)
                if !newline {
                    crate::parser::document::helpers::skip_whitespace_no_tabs(source)?;
                    if source.current() == Some('-') {
                        let state = source.save_state();
                        source.next();
                        if let Some(c) = source.current() {
                            if c.is_whitespace() || c == '\n' {
                                // This is a sequence item on the same line as a key - error
                                source.restore_state(state);
                                return Err(parse_error(
                                    source,
                                    "Sequence cannot start on same line as mapping key",
                                ));
                            }
                        }
                        source.restore_state(state);
                    }
                }

                let next_indent = source.get_current_indent_level();
                if next_indent > indent_level && newline {
                    pairs.push((
                        key_node,
                        crate::parser::document::parse_document_contents(
                            source,
                            next_indent,
                            directives,
                        )?,
                    ));
                    continue;
                } else {
                    let mut value_node = parse_value(source, directives)?;
                    if let Node::Str(ref mut s, ref mut qt, ref mut style) = value_node {
                        if matches!(*qt, QuoteType::Single | QuoteType::Double)
                            && is_plain_safe_value(s)
                        {
                            if !matches!(*style, BlockStyle::Literal) {
                                *style = BlockStyle::None;
                            }
                            *qt = QuoteType::Unquoted;
                        }
                    }
                    last_was_nested = matches!(value_node, Node::Anchored(_, _))
                        || matches!(value_node, Node::Str(_, _, BlockStyle::Literal));
                    pairs.push((key_node, value_node));
                }
            }
            c if c.is_whitespace() => {
                source.next();
                continue;
            }
            _ => break,
        }
        if !last_was_nested {
            crate::utils::skip_until_newline(source);
            crate::parser::document::helpers::skip_whitespace_no_tabs(source)?;
        }
    }
    Ok(Node::Mapping(pairs))
}
