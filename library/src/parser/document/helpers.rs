//! Module: parser/document/helpers.rs

use crate::constants::*;
use crate::error::messages::*;
use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::nodes::node::Node::Document;
use crate::nodes::node::{BlockStyle, QuoteType};
use crate::parser::document::context::ParsingContext;
use crate::parser::document::error_builder::forbidden_error;
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

/// Unified indentation validation using parsing context.
///
/// This function provides context-aware validation of indentation, particularly
/// for tab characters which are forbidden in certain contexts per YAML 1.2 spec.
///
/// Tabs are forbidden when:
/// - In block context (not flow collections like [], {})
/// - Used as indentation (after newlines, before content)
///
/// Tabs are allowed when:
/// - In flow context (inside [], {}, quoted strings)
/// - Part of string content (not indentation)
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `ctx` - The current parsing context (determines validation rules)
///
/// # Returns
///
/// `Ok(())` if indentation is valid, `Err(String)` if tabs found in forbidden context
///
/// # Example
///
/// ```ignore
/// let ctx = ParsingContext::new(0);
/// validate_indentation(source, &ctx)?;
/// ```
pub(crate) fn validate_indentation(
    source: &mut dyn ISource,
    ctx: &ParsingContext,
) -> Result<(), String> {
    // In flow context, tabs are allowed (different whitespace rules)
    if ctx.in_flow {
        return Ok(());
    }

    // Only validate tabs when we're at indentation position (after newline)
    if !ctx.should_validate_tab_indentation() {
        return Ok(());
    }

    // From indentation position, any tab encountered before actual content is forbidden
    while let Some(c) = source.current() {
        if c == CHAR_TAB {
            return Err(forbidden_error(source, "Tabs", "as indentation in YAML"));
        } else if c == CHAR_SPACE {
            source.next();
            continue;
        } else if c == CHAR_NEWLINE || c == CHAR_CARRIAGE_RETURN {
            // Truly blank line
            return Ok(());
        } else {
            // Non-space content reached; stop validation
            break;
        }
    }

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

/// Skips whitespace with context-aware tab validation.
///
/// This validates indentation first (before consuming), then skips whitespace.
/// This order ensures tabs in indentation are caught before being consumed.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `ctx` - The current parsing context
///
/// # Returns
///
/// `Ok(())` if successful, `Err(String)` if tabs found in forbidden context
pub(crate) fn skip_whitespace_with_context(
    source: &mut dyn ISource,
    ctx: &ParsingContext,
) -> Result<(), String> {
    // Validate BEFORE consuming whitespace
    validate_indentation(source, ctx)?;
    skip_whitespace(source);
    Ok(())
}

/// Skips whitespace but returns an error if tabs are found as line indentation.
///
/// **DEPRECATED**: Use `skip_whitespace_with_context()` with proper ParsingContext instead.
/// This function is kept for backward compatibility during refactoring.
///
/// Handles newlines and tracks whether tabs appear after them (which would be indentation).
/// Per YAML 1.2 spec, tabs cannot be used for indentation.
/// Note: This function assumes it may be called after a newline has already been consumed,
/// so it starts by assuming we're at the beginning of a line and validates from there.
pub(crate) fn skip_whitespace_no_tabs(source: &mut dyn ISource) -> Result<(), String> {
    let mut found_tab_after_newline = false;
    let mut after_newline = true; // Assume we start after a newline (caller consumed it)

    while let Some(c) = source.current() {
        if c == '\n' || c == '\r' {
            // Consume newline and mark that we're at line start
            source.next();
            after_newline = true;
            found_tab_after_newline = false; // Reset for new line
        } else if c == '\t' {
            if after_newline {
                // Tab after newline = indentation = forbidden
                found_tab_after_newline = true;
            }
            source.next();
        } else if c == ' ' {
            source.next();
        } else {
            // Found actual non-whitespace content
            if found_tab_after_newline {
                // Tabs were used as indentation - error
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

/// Validate that there are no tabs in the leading whitespace at line start.
///
/// **DEPRECATED**: Use `validate_indentation()` with proper ParsingContext instead.
/// This function is kept for backward compatibility during refactoring.
///
/// This should be called after processing a newline, before any content
pub(crate) fn validate_no_tab_indentation(source: &mut dyn ISource) -> Result<(), String> {
    // Create a temporary context for validation (assumes block context after newline)
    let mut ctx = ParsingContext::new(source.get_current_indent_level());
    ctx.mark_newline_consumed();
    validate_indentation(source, &ctx)
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
    let mut at_line_start = true;
    loop {
        match source.current() {
            Some(c) => {
                // Check for document markers at line start
                if at_line_start && (c == '-' || c == '.') {
                    if peek_ahead_for_document_start_end(source, c) {
                        return Err(parse_error(
                            source,
                            "Document marker found inside quoted string - quotes must be closed before document markers",
                        ));
                    }
                }

                at_line_start = c == '\n';

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

    // Skip tags (!!tag or !tag) if present
    while source.current() == Some('!') {
        source.next(); // Skip first !
        // Check for second ! (!!tag)
        if source.current() == Some('!') {
            source.next(); // Skip second !
        }
        // Skip tag name (alphanumeric, -, _, . but NOT colon - that would consume the mapping separator)
        while let Some(c) = source.current() {
            if c.is_alphanumeric() || c == CHAR_DASH || c == '_' || c == CHAR_DOT {
                source.next();
            } else {
                break;
            }
        }
        // Skip whitespace after tag
        while matches!(source.current(), Some(CHAR_SPACE) | Some(CHAR_TAB)) {
            source.next();
        }
    }

    // Skip anchors (&name) if present
    if source.current() == Some(CHAR_AMPERSAND) {
        source.next(); // Skip &
        // Skip anchor name (but NOT colon - anchor names with colons are allowed per spec,
        // but in this context we're looking for mapping keys, so stop at colon)
        while let Some(c) = source.current() {
            if c.is_alphanumeric() || c == CHAR_DASH || c == '_' {
                source.next();
            } else {
                break;
            }
        }
        // Skip whitespace after anchor
        while matches!(source.current(), Some(CHAR_SPACE) | Some(CHAR_TAB)) {
            source.next();
        }
    }

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
                CHAR_NEWLINE | CHAR_CARRIAGE_RETURN => {
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
    // Check for anchors, aliases, or tags at the start of the key
    let key_node = if matches!(
        source.current(),
        Some(CHAR_AMPERSAND) | Some(CHAR_ASTERISK) | Some('!')
    ) {
        // Use parse_value to handle anchors/aliases/tags properly
        crate::parser::document::value::parse_value(source, directives)?
    } else {
        // Original collection-based parsing for plain scalars
        let raw = collect_until(source, |c| {
            c == CHAR_COLON || c == CHAR_NEWLINE || c == CHAR_CARRIAGE_RETURN
        });

        // Check if we stopped at a colon or newline/carriage return
        if source.current() == Some(CHAR_NEWLINE) || source.current() == Some(CHAR_CARRIAGE_RETURN)
        {
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

        match raw.trim() {
            v if v.starts_with(CHAR_HASH) => {
                Node::Str(v.to_string(), QuoteType::Unquoted, BlockStyle::None)
            }
            v => crate::parser::document::scalar::parse_scalar(v, directives),
        }
    };

    // Now consume the colon and check for newline
    let mut newline = false;
    source.next(); // consume the colon
    skip_whitespace(source); // Tabs OK here - not indentation (same line as colon)
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
                // Create temporary context for validation (assume block mapping at current indent)
                let mut temp_ctx = crate::parser::document::context::ParsingContext::new(
                    source.get_current_indent_level()
                );
                temp_ctx.mark_newline_consumed();
                skip_whitespace_with_context(source, &temp_ctx)?;
            }
        }
    }

    Ok((key_node, newline))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::parser::document::context::{CollectionType, ParsingContext};

    #[test]
    fn test_validate_indentation_block_context_with_tab() {
        // Tab in block context after newline should error
        let mut source = Buffer::new(b"\t  content");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let result = validate_indentation(&mut source, &ctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Tabs") && err.contains("not allowed"),
            "Error: {}",
            err
        );
    }

    #[test]
    fn test_validate_indentation_block_context_no_tab() {
        // Spaces in block context are OK
        let mut source = Buffer::new(b"  content");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let result = validate_indentation(&mut source, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_indentation_flow_context_with_tab() {
        // Tab in flow context is allowed
        let mut source = Buffer::new(b"\t  content");
        let ctx = ParsingContext::new(0).child_flow_context(CollectionType::FlowSequence);

        let result = validate_indentation(&mut source, &ctx);
        assert!(result.is_ok(), "Tabs should be allowed in flow context");
    }

    #[test]
    fn test_validate_indentation_not_after_newline() {
        // Tab not immediately after newline (content found) - should not validate
        let mut source = Buffer::new(b"\tcontent");
        let ctx = ParsingContext::new(0);
        // Don't mark newline consumed - simulates tab in middle of content

        let result = validate_indentation(&mut source, &ctx);
        assert!(result.is_ok(), "Tabs OK when not at indentation position");
    }

    #[test]
    fn test_validate_indentation_blank_line() {
        // Tab at line start is indentation even if followed by newline
        let mut source = Buffer::new(b"\t\n");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let result = validate_indentation(&mut source, &ctx);
        // Should error - tab is still indentation even on "blank" lines
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_indentation_actual_blank_line() {
        // Truly blank line (just newline, no tabs) should be OK
        let mut source = Buffer::new(b"\n");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let result = validate_indentation(&mut source, &ctx);
        assert!(result.is_ok(), "Blank lines without tabs should be OK");
    }

    #[test]
    fn test_validate_indentation_spaces_then_tab() {
        // Spaces followed by tab in indentation
        let mut source = Buffer::new(b"  \tcontent");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let result = validate_indentation(&mut source, &ctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Tabs") && err.contains("not allowed"),
            "Error: {}",
            err
        );
    }

    #[test]
    fn test_validate_no_tab_indentation_backward_compat() {
        // Test backward compatibility wrapper
        let mut source = Buffer::new(b"\tcontent");

        let result = validate_no_tab_indentation(&mut source);
        assert!(result.is_err());
    }

    #[test]
    fn test_skip_whitespace_with_context_block() {
        // Test the new wrapper function
        let mut source = Buffer::new(b"  content");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let result = skip_whitespace_with_context(&mut source, &ctx);
        assert!(result.is_ok());
        assert_eq!(source.current(), Some('c'));
    }

    #[test]
    fn test_skip_whitespace_with_context_tab_error() {
        let mut source = Buffer::new(b"\tcontent");
        let mut ctx = ParsingContext::new(0);
        ctx.mark_newline_consumed();

        let result = skip_whitespace_with_context(&mut source, &ctx);
        assert!(result.is_err(), "Should error on tab in block indentation");
    }

    #[test]
    fn test_skip_whitespace_with_context_flow() {
        // Tabs OK in flow context
        let mut source = Buffer::new(b"\tcontent");
        let ctx = ParsingContext::new(0).child_flow_context(CollectionType::FlowMapping);

        let result = skip_whitespace_with_context(&mut source, &ctx);
        assert!(result.is_ok());
    }
}
