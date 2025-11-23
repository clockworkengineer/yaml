//! Module: parser/document/helpers.rs

use crate::constants::*;
use crate::error::messages::*;
use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::nodes::node::Node::Document;
use crate::nodes::node::{BlockStyle, QuoteType};
use crate::utils::*;

/// Creates a formatted error message with current parser context information.
///
/// Generates an error message that includes the current character being parsed
/// and the current indentation level to help with debugging parsing issues.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `msg` - The base error message to include
///
/// # Returns
///
/// A formatted error string with context information
pub(crate) fn parse_error(source: &mut dyn ISource, msg: &str) -> String {
    let current = match source.current() {
        Some(c) => c.to_string(),
        None => STR_EOF.to_string(),
    };
    format!(
        "{} (current: '{}', indent: {})",
        msg,
        current,
        source.get_current_indent_level()
    )
}

/// Validates that tabs are not used for indentation at current position.
/// According to YAML 1.2 spec, tabs cannot be used for indentation.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
///
/// # Returns
///
/// `Ok(())` if no tabs in indentation, `Err(String)` if tabs found
pub(crate) fn validate_no_tabs_in_indentation(source: &mut dyn ISource) -> Result<(), String> {
    let state = source.save_state();

    // Check from current position for tabs before non-whitespace
    while let Some(c) = source.current() {
        if c == CHAR_TAB {
            source.restore_state(state);
            return Err(parse_error(source, "Tabs cannot be used for indentation in YAML"));
        }
        if c == CHAR_SPACE {
            source.next();
            continue;
        }
        // Non-whitespace found, no tabs in indentation
        break;
    }

    source.restore_state(state);
    Ok(())
}

/// Skips whitespace characters in the source.
///
/// Advances the source position past all consecutive whitespace characters
/// as defined by the source's is_whitespace method.
/// Note: This does NOT validate tabs - tabs are allowed in some contexts.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
pub(crate) fn skip_whitespace(source: &mut dyn ISource) {
    while let Some(c) = source.current() {
        if source.is_whitespace(c) {
            source.next();
        } else {
            break;
        }
    }
}

/// Skips whitespace but returns an error if tabs are found as line indentation
/// This should be called after consuming a newline, where tabs would be indentation
pub(crate) fn skip_whitespace_no_tabs(source: &mut dyn ISource) -> Result<(), String> {
    let mut found_tab_before_content = false;
    
    while let Some(c) = source.current() {
        if c == '\t' {
            // Mark that we found a tab - we'll error if followed by content
            found_tab_before_content = true;
            source.next();
        } else if c == ' ' {
            source.next();
        } else if c == '\n' || c == '\r' {
            // Blank line - tabs don't matter here
            return Ok(());
        } else {
            // Found actual content after whitespace
            if found_tab_before_content {
                // Tabs before content = indentation = forbidden
                return Err(crate::parser::document::parse_error(
                    source,
                    "Tabs are not allowed as indentation in YAML",
                ));
            }
            break;
        }
    }
    Ok(())
}

/// Validate that there are no tabs in the leading whitespace at line start
/// This should be called after processing a newline, before any content
pub(crate) fn validate_no_tab_indentation(source: &mut dyn ISource) -> Result<(), String> {
    // Only check if we're at the start of a line (column 0 or only whitespace so far)
    let state = source.save_state();

    // Check characters from current position forward
    while let Some(c) = source.current() {
        if c == '\t' {
            // Found a tab - this is invalid indentation
            source.restore_state(state);
            return Err(crate::parser::document::parse_error(
                source,
                "Tabs are not allowed as indentation in YAML",
            ));
        } else if c == ' ' {
            source.next();
        } else if c == '\n' || c == '\r' {
            // Blank line - tabs would be OK here (no content)
            source.restore_state(state);
            return Ok(());
        } else {
            // Found content
            break;
        }
    }

    source.restore_state(state);
    Ok(())
}

/// Determines if a node represents blank or empty content.
///
/// Checks various node types to determine if they should be considered
/// blank (None, empty arrays, empty strings, comments, etc.).
///
/// # Arguments
///
/// * `node` - A reference to the Node to check
///
/// # Returns
///
/// true if the node is considered blank, false otherwise
pub(crate) fn node_is_blank(node: &Node) -> bool {
    match node {
        Node::None => true,
        Node::Array(items) => items.is_empty(),
        Node::Mapping(_pairs) => false,
        Document(nodes) => nodes.iter().all(node_is_blank),
        Node::Str(s, _, _) => s.is_empty(),
        Node::Comment(_) => true,
        Node::Anchored(inner, _name) => node_is_blank(inner),
        Node::Alias(_name) => false,
        _ => false,
    }
}

/// Parses a quoted scalar value (single or double quoted) from the source.
///
/// Handles both single and double quoted strings, processing escape sequences
/// and handling multiline strings correctly. Returns the complete quoted string
/// including the quote characters.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
///
/// # Returns
///
/// Result containing the complete quoted string or an error string
pub(crate) fn parse_quoted_scalar(source: &mut dyn ISource) -> Result<String, String> {
    let quote = match source.current() {
        Some(c) if c == CHAR_SINGLE_QUOTE || c == CHAR_DOUBLE_QUOTE => c,
        Some(other) => {
            let msg = ERR_EXPECT_QUOTE_FORMAT.replace("{}", &other.to_string());
            return Err(parse_error(source, &msg).to_string());
        }
        None => return Err(parse_error(source, ERR_UNEXPECTED_EOF_EXPECTING_QUOTE)),
    };
    let mut out = String::new();
    out.push(quote);
    source.next();

    let mut prev_was_backslash = false;
    loop {
        match source.current() {
            Some(c) => {
                out.push(c);
                source.next();

                if c == quote {
                    if quote == CHAR_SINGLE_QUOTE {
                        if source.current() == Some(CHAR_SINGLE_QUOTE) {
                            out.push(CHAR_SINGLE_QUOTE);
                            source.next();
                            continue;
                        } else {
                            break;
                        }
                    } else if prev_was_backslash {
                        prev_was_backslash = false;
                        continue;
                    } else {
                        break;
                    }
                }

                if quote == CHAR_DOUBLE_QUOTE {
                    if c == CHAR_BACKSLASH {
                        prev_was_backslash = !prev_was_backslash;
                    } else {
                        prev_was_backslash = false;
                    }
                }
            }
            None => {
                return Err(parse_error(source, ERR_UNTERMINATED_QUOTED_FLOW));
            }
        }
    }

    // Validate escape sequences in double-quoted strings
    if quote == CHAR_DOUBLE_QUOTE && out.len() >= 2 {
        let inner = &out[1..out.len() - 1];
        if let Err(e) = crate::utils::validate_double_quoted_escapes(inner) {
            return Err(parse_error(source, &e));
        }
    }

    // Validate comment spacing after quoted scalar
    // After closing quote, if next char is #, it's invalid (needs whitespace)
    if source.current() == Some(CHAR_HASH) {
        return Err(parse_error(
            source,
            "Comment indicator (#) must be preceded by whitespace",
        ));
    }

    Ok(out)
}

/// Peeks ahead to check if the current position represents a document start/end marker.
///
/// Looks for document separators (---) or end markers (...) by checking if the
/// current character is repeated three times consecutively.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `c` - The character to check for repetition (usually '-' or '.')
///
/// # Returns
///
/// true if a document marker is found, false otherwise
pub(crate) fn peek_ahead_for_document_start_end(source: &mut dyn ISource, c: char) -> bool {
    if source.current() != Some(c) {
        return false;
    }
    let state = source.save_state();
    source.next();
    if source.current() != Some(c) {
        source.restore_state(state);
        return false;
    }
    source.next();
    if source.current() != Some(c) {
        source.restore_state(state);
        return false;
    }
    source.restore_state(state);
    true
}

/// Peeks ahead to determine if the current content represents a mapping key.
///
/// Looks for a colon (:) character that would indicate the current content
/// should be parsed as a mapping key rather than a standalone value.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
///
/// # Returns
///
/// true if a mapping key pattern is detected, false otherwise
pub(crate) fn peek_ahead_for_mapping_key(source: &mut dyn ISource) -> bool {
    let mut found = false;
    let state = source.save_state();

    // Handle quoted strings specially - they can span multiple lines
    if matches!(
        source.current(),
        Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE)
    ) {
        let quote = source.current().unwrap();
        source.next(); // Skip opening quote

        // Scan until we find the closing quote
        let mut prev_was_backslash = false;
        while let Some(c) = source.current() {
            source.next(); // Move past current character

            if c == quote {
                if quote == CHAR_SINGLE_QUOTE {
                    // Check for doubled single quote (escape mechanism)
                    if source.current() == Some(CHAR_SINGLE_QUOTE) {
                        source.next(); // Skip the second quote
                        prev_was_backslash = false;
                        continue;
                    } else {
                        break; // Found true closing quote
                    }
                } else if !prev_was_backslash {
                    break; // Found true closing double quote
                }
            }

            if quote == CHAR_DOUBLE_QUOTE && c == CHAR_BACKSLASH {
                prev_was_backslash = !prev_was_backslash;
            } else {
                prev_was_backslash = false;
            }
        }

        // After the quoted string, skip whitespace (not newlines) and check for colon
        while let Some(c) = source.current() {
            if c == CHAR_SPACE || c == CHAR_TAB {
                source.next();
            } else {
                break;
            }
        }

        // Check for colon (not after newline)
        if source.current() == Some(CHAR_COLON) {
            found = true;
        }
    } else {
        // Non-quoted case: look for colon before newline
        while let Some(c) = source.current() {
            match c {
                CHAR_COLON => {
                    found = true;
                    break;
                }
                CHAR_NEWLINE => {
                    break;
                }
                _ => {
                    if source.more() {
                        source.next();
                    }
                }
            }
        }
    }

    source.restore_state(state);
    found
}

/// Parses a mapping key and determines if it has an explicit complex structure.
///
/// Handles both simple keys and complex keys (like sequences or mappings used as keys).
/// Returns the parsed key node and a boolean indicating if it's a complex key.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `directives` - Directive context for version-aware parsing
///
/// # Returns
///
/// Result containing a tuple of (key_node, is_complex_key) or an error string
pub(crate) fn parse_mapping_key(
    source: &mut dyn ISource,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<(Node, bool), String> {
    let raw = collect_until(source, |c| {
        c == CHAR_COLON || c == CHAR_NEWLINE || c == CHAR_CARRIAGE_RETURN
    });

    // Check if we stopped at a colon or newline/carriage return
    if source.current() == Some(CHAR_NEWLINE) || source.current() == Some(CHAR_CARRIAGE_RETURN) {
        // We reached end of line without finding a colon - invalid mapping key
        return Err(parse_error(
            source,
            "Mapping key must be followed by a colon",
        ));
    }

    // Check if we hit EOF without a colon
    if source.current().is_none() {
        // EOF without colon - could be valid scalar, not a mapping
        // But we're in parse_mapping_key, so this shouldn't happen
        // Let the caller handle it
        return Err(parse_error(
            source,
            "Unexpected end of input in mapping key",
        ));
    }

    let mut newline = false;
    source.next(); // consume the colon
    skip_whitespace(source);  // Tabs OK here - not indentation (same line as colon)
    if let Some(c) = source.current() {
        if c == CHAR_HASH {
            consume_inline_comment_and_newline(source);
            newline = true;
        } else {
            // Handle Windows line endings (\r\n) and Unix (\n)
            if c == CHAR_CARRIAGE_RETURN {
                source.next();
                newline = true;
            }
            if source.current() == Some(CHAR_NEWLINE) {
                source.next();
                newline = true;
            }
            if newline {
                // After newline, validate no tabs in indentation
                skip_whitespace_no_tabs(source)?;
            }
        }
    }

    match raw.trim() {
        v if v.starts_with(CHAR_HASH) => Ok((
            Node::Str(v.to_string(), QuoteType::Unquoted, BlockStyle::None),
            newline,
        )),
        v => Ok((
            crate::parser::document::scalar::parse_scalar(v, directives),
            newline,
        )),
    }
}

/// Parses a comment line from the source.
///
/// Consumes a comment starting with '#' and returns the comment text
/// without the hash character and with trailing whitespace trimmed.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
///
/// # Returns
///
/// The comment text as a String
pub(crate) fn parse_comment(source: &mut dyn ISource) -> String {
    source.next();
    read_line_trimmed_into_string(source)
}

/// Validates that a comment indicator (#) is preceded by whitespace or at start of line.
///
/// According to YAML spec, # can only start a comment if preceded by whitespace
/// or if it's at the beginning of a line.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `prev_char` - The character that preceded the current position
///
/// # Returns
///
/// Result with Ok(()) if valid, Err with error message if invalid
pub(crate) fn validate_comment_spacing(
    source: &mut dyn ISource,
    prev_char: Option<char>,
) -> Result<(), String> {
    if source.current() == Some('#') {
        // # must be preceded by whitespace, newline, or be at start
        if let Some(prev) = prev_char {
            if !prev.is_whitespace() && prev != '\n' && prev != '\r' {
                return Err(parse_error(
                    source,
                    "Comment indicator (#) must be preceded by whitespace",
                ));
            }
        }
    }
    Ok(())
}

/// Converts a node to its inline string representation for display.
///
/// Similar to the utility function but specifically tailored for parser
/// context. Provides compact string representations for debugging.
///
/// # Arguments
///
/// * `node` - A reference to the Node to convert
///
/// # Returns
///
/// A String containing the inline representation
pub(crate) fn node_to_inline_string(node: &Node) -> String {
    crate::utils::node_to_inline_string(node)
}
