//! Helper functions for parsing explicit mapping keys (? indicator)

use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::document::error_builder::syntax_error;

/// Checks if the current position starts an explicit key (?)
#[allow(dead_code)]
pub(crate) fn is_explicit_key_start(source: &mut dyn ISource) -> bool {
    source.current() == Some('?')
}

/// Parses an explicit mapping key-value pair
///
/// Format: ? key\n  : value
/// The ? must be at the start of a line
/// Returns (key_node, value_node)
#[allow(dead_code)]
pub(crate) fn parse_explicit_mapping_entry(
    source: &mut dyn ISource,
    indent_level: usize,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<(Node, Node), String> {
    // Skip the '?' indicator
    if source.current() != Some('?') {
        return Err(syntax_error(source, "Expected '?' for explicit key"));
    }
    source.next();

    // Skip whitespace after ?
    crate::parser::document::helpers::skip_whitespace(source);

    // Parse the key
    let key_node = if source.current() == Some('\n') {
        // Empty explicit key
        source.next();
        crate::parser::document::helpers::skip_whitespace(source);
        let key_indent = source.get_current_indent_level();
        crate::parser::document::parse_document_contents(source, key_indent, directives)?
    } else {
        // Key on same line as ?
        crate::parser::document::parse_value(source, directives)?
    };

    // Look for the : indicator
    crate::parser::document::helpers::skip_whitespace(source);

    let value_node = if source.current() == Some(':') {
        source.next();
        crate::parser::document::helpers::skip_whitespace(source);

        if source.current() == Some('\n') {
            // Value on next line
            source.next();
            crate::parser::document::helpers::skip_whitespace(source);
            let value_indent = source.get_current_indent_level();
            if value_indent > indent_level {
                crate::parser::document::parse_document_contents(source, value_indent, directives)?
            } else {
                Node::None
            }
        } else {
            // Value on same line
            crate::parser::document::parse_value(source, directives)?
        }
    } else {
        // No value indicator, key only
        Node::None
    };

    Ok((key_node, value_node))
}
