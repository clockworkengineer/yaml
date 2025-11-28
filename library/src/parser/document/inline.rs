//! Module: parser/document/inline.rs

use crate::constants::*;
use crate::error::messages::*;
use crate::io::traits::ISource;
use crate::nodes::node::{BlockStyle, Node, QuoteType};
use crate::parser::document::helpers::{
    parse_error, parse_quoted_scalar, skip_whitespace_no_tabs, validate_comment_spacing,
};
use crate::parser::document::scalar::parse_scalar;
use crate::parser::document::value::parse_value;
use crate::utils::*;

/// Collects a plain scalar in flow context, handling line folding.
/// Newlines followed by spaces fold into a single space per YAML spec.
fn collect_flow_scalar<F>(source: &mut dyn ISource, mut stop_pred: F) -> String
where
    F: FnMut(char) -> bool,
{
    let mut out = String::new();
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 100_000;

    while let Some(c) = source.current() {
        // Check stop condition first (before consuming)
        if stop_pred(c) {
            break;
        }

        // Allow quoted scalars to be parsed as a single item
        if c == '\'' || c == '"' {
            let quote_char = c;
            let mut quoted = String::new();
            source.next();
            while let Some(qc) = source.current() {
                if qc == quote_char {
                    source.next();
                    break;
                }
                quoted.push(qc);
                source.next();
            }
            out.push_str(&quoted);
            continue;
        }

        if c == '\n' || c == '\r' {
            // Handle newline with line folding
            source.next();
            // Skip the following indentation spaces
            while let Some(next_c) = source.current() {
                if next_c == ' ' || next_c == '\t' {
                    source.next();
                } else {
                    break;
                }
            }
            // Check if we hit a stop character after the newline+spaces
            if let Some(next_c) = source.current() {
                if stop_pred(next_c) {
                    break;
                }
            }
            // Add a single space for the folded line (if we have content)
            if !out.is_empty() && !out.ends_with(' ') {
                out.push(' ');
            }
            continue;
        }

        out.push(c);
        source.next();

        iterations += 1;
        if iterations >= MAX_ITERATIONS {
            break;
        }
    }
    out
}

/// Parses an inline YAML set enclosed in curly braces {} without colons.
///
/// Handles comma-separated values within braces for set syntax like {item1, item2, item3}.
/// This is used when the inline mapping parser detects no colons in the content.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `directives` - Directive context for tag resolution
///
/// # Returns
///
/// Result containing a Mapping Node with null values (for set coercion) or an error string
pub(crate) fn parse_inline_set(
    source: &mut dyn ISource,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
    let mut pairs: Vec<(Node, Node)> = Vec::new();
    let mut iterations = 0;
    const MAX_ITEMS: usize = 10_000;

    source.next(); // Skip the opening '{'
    skip_whitespace_no_tabs(source)?;

    if source.current() == Some(CHAR_RBRACE) {
        source.next();
        return Ok(Node::Mapping(pairs)); // Empty set as empty mapping
    }

    loop {
        // Prevent infinite loop
        iterations += 1;
        if iterations >= MAX_ITEMS {
            return Err(parse_error(
                source,
                "Flow set too large or malformed - possible infinite loop",
            ));
        }

        // Skip whitespace before checking for items
        skip_whitespace_no_tabs(source)?;

        // Check for closing brace (handles trailing comma case)
        if source.current() == Some(CHAR_RBRACE) {
            source.next();
            break;
        }

        let item_node = match source.current() {
            Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                let raw = parse_quoted_scalar(source)?;
                parse_scalar(raw.trim(), directives)
            }
            Some(CHAR_LBRACE) => parse_inline_mapping(source, directives)?,
            Some(CHAR_LBRACKET) => parse_inline_sequence(source, directives)?,
            Some(_) => {
                let val = collect_flow_scalar(source, |c| {
                    c == CHAR_COMMA || c == CHAR_RBRACE || c == CHAR_HASH
                });
                let trimmed = val.trim();
                if trimmed.is_empty() {
                    return Err(parse_error(source, "Empty item in inline set"));
                }
                parse_scalar(trimmed, directives)
            }
            None => return Err(parse_error(source, "Unexpected EOF in inline set")),
        };

        // Add item as a key with null value (set format)
        pairs.push((item_node, Node::None));

        skip_whitespace_and_comments_validate_tabs(source)?;

        match source.current() {
            Some(CHAR_COMMA) => {
                source.next();
                skip_whitespace_no_tabs(source)?;
                // Check for trailing comma (comma followed by closing brace)
                if source.current() == Some(CHAR_RBRACE) {
                    source.next();
                    break;
                }
                continue;
            }
            Some(CHAR_RBRACE) => {
                source.next();
                break;
            }
            Some(c) => {
                return Err(parse_error(
                    source,
                    &format!("Unexpected character in inline set: {}", c),
                ));
            }
            None => return Err(parse_error(source, "Unexpected EOF in inline set")),
        }
    }

    Ok(Node::Mapping(pairs))
}

/// Parses an inline YAML mapping or set enclosed in curly braces {}.
///
/// First attempts to parse as a mapping with key-value pairs. If no colons
/// are found, parses as an inline set with comma-separated items.
/// Supports empty mappings/sets and nested structures.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `directives` - Directive context for tag resolution
///
/// # Returns
///
/// Result containing a Mapping Node or an error string
pub(crate) fn parse_inline_mapping(
    source: &mut dyn ISource,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
    // Save the current position to potentially backtrack
    let saved_state = source.save_state();

    // Look ahead to see if this is a set (no colons) or a mapping (has colons)
    let mut has_colons = false;
    let mut brace_depth = 0;
    let mut bracket_depth = 0;
    let mut in_quotes = false;
    let mut quote_char = '\0';
    let mut iterations = 0;
    const MAX_LOOKAHEAD: usize = 10_000;

    source.next(); // Skip opening brace

    while let Some(c) = source.current() {
        // Prevent infinite loop in lookahead
        iterations += 1;
        if iterations >= MAX_LOOKAHEAD {
            // If we've scanned too far, assume it's malformed - restore and try as mapping
            source.restore_state(saved_state);
            return parse_inline_mapping_with_colons(source, directives);
        }

        match c {
            CHAR_RBRACE if brace_depth == 0 && !in_quotes => break,
            CHAR_LBRACE if !in_quotes => brace_depth += 1,
            CHAR_RBRACE if !in_quotes => brace_depth -= 1,
            CHAR_LBRACKET if !in_quotes => bracket_depth += 1,
            CHAR_RBRACKET if !in_quotes => bracket_depth -= 1,
            CHAR_SINGLE_QUOTE | CHAR_DOUBLE_QUOTE if !in_quotes => {
                in_quotes = true;
                quote_char = c;
            }
            c if in_quotes && c == quote_char => {
                in_quotes = false;
                quote_char = '\0';
            }
            CHAR_COLON if !in_quotes && brace_depth == 0 && bracket_depth == 0 => {
                // In flow context, a colon can be a mapping separator if:
                // 1. It's preceded by whitespace, quotes, or flow collection delimiters
                // 2. OR it's followed by whitespace, end markers, or flow collection delimiters
                // But NOT if it's part of a URL-like pattern (e.g., http://)

                source.next(); // Move to next character
                let next_char = source.current();

                // Check for http:// or similar patterns - if followed by //, it's not a separator
                if next_char == Some('/') {
                    source.next();
                    if source.current() == Some('/') {
                        // This is a URL-like pattern (://), not a mapping separator
                        source.next();
                        continue;
                    }
                    // Just a single slash - this could be a separator followed by value
                }

                // If followed by whitespace or flow markers, it's a separator
                // If followed by non-whitespace, it's still a separator in flow context (e.g., key:value)
                // The only exception is the URL pattern we checked above
                has_colons = true;
                break; // Found a mapping separator colon
            }
            // Skip whitespace and newlines in flow context
            c if !in_quotes && (c == ' ' || c == '\t' || c == '\n' || c == '\r') => {}
            _ => {}
        }
        source.next();
    }

    // Restore position and parse accordingly
    source.restore_state(saved_state);

    if has_colons {
        parse_inline_mapping_with_colons(source, directives)
    } else {
        parse_inline_set(source, directives)
    }
}
/// Parses an inline YAML mapping with key-value pairs (original implementation).
fn parse_inline_mapping_with_colons(
    source: &mut dyn ISource,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
    let mut pairs: Vec<(Node, Node)> = Vec::new();
    let mut iterations = 0;
    const MAX_PAIRS: usize = 10_000;

    source.next();
    skip_whitespace_no_tabs(source)?;

    if source.current() == Some(CHAR_RBRACE) {
        source.next();
        return Ok(Node::Mapping(pairs));
    }

    loop {
        // Prevent infinite loop
        iterations += 1;
        if iterations >= MAX_PAIRS {
            return Err(parse_error(
                source,
                "Flow mapping too large or malformed - possible infinite loop",
            ));
        }
        // Skip whitespace, newlines, and comments before parsing key
        skip_whitespace_and_comments_validate_tabs(source)?;

        // Check for closing brace after whitespace (handles trailing comma case)
        if source.current() == Some(CHAR_RBRACE) {
            source.next();
            break;
        }

        // If we're at None (EOF) inside a flow mapping, that's an error
        if source.current().is_none() {
            return Err(parse_error(source, ERR_EOF_INLINE_MAPPING));
        }

        let key_node = match source.current() {
            Some('?') => {
                // Explicit key indicator
                source.next();
                skip_whitespace_and_comments_validate_tabs(source)?;

                // Check what follows the ?
                let key = match source.current() {
                    Some(CHAR_COLON) | Some(CHAR_COMMA) | Some(CHAR_RBRACE) | None => {
                        // ? followed by : or , or } or EOF means null key
                        Node::None
                    }
                    Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                        let raw = parse_quoted_scalar(source)?;
                        parse_scalar(raw.trim(), directives)
                    }
                    Some(CHAR_LBRACKET) => parse_inline_sequence(source, directives)?,
                    Some(CHAR_LBRACE) => parse_inline_mapping(source, directives)?,
                    _ => {
                        // Parse regular scalar key with line folding
                        let collected = collect_flow_scalar(source, |c| {
                            c == CHAR_COLON || c == CHAR_RBRACE || c == CHAR_COMMA
                        });
                        parse_scalar(collected.trim(), directives)
                    }
                };

                skip_whitespace_and_comments_validate_tabs(source)?;

                // After explicit key, colon is optional
                // If there's a colon, consume it and move to value parsing
                // If no colon, the value is implicitly null
                if source.current() == Some(CHAR_COLON) {
                    source.next();
                }

                key
            }
            Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                let raw = parse_quoted_scalar(source)?;
                skip_whitespace_and_comments_validate_tabs(source)?;
                let key_node = parse_scalar(raw.trim(), directives);

                // Check if there's a colon for the value
                if source.current() == Some(CHAR_COLON) {
                    source.next();
                } else if source.current() == Some(CHAR_COMMA)
                    || source.current() == Some(CHAR_RBRACE)
                {
                    // No colon means implicit null value
                } else {
                    return Err(parse_error(source, ERR_EXPECT_COLON_INLINE_MAPPING));
                }

                key_node
            }
            Some(CHAR_LBRACKET) => {
                // Flow sequence as key
                let key = parse_inline_sequence(source, directives)?;
                skip_whitespace_and_comments_validate_tabs(source)?;

                // Check if there's a colon for the value
                if source.current() == Some(CHAR_COLON) {
                    source.next();
                } else if source.current() == Some(CHAR_COMMA)
                    || source.current() == Some(CHAR_RBRACE)
                {
                    // No colon means implicit null value
                } else {
                    return Err(parse_error(source, ERR_EXPECT_COLON_INLINE_MAPPING));
                }

                key
            }
            Some(CHAR_LBRACE) => {
                // Flow mapping as key (nested)
                let key = parse_inline_mapping(source, directives)?;
                skip_whitespace_and_comments_validate_tabs(source)?;

                // Check if there's a colon for the value
                if source.current() == Some(CHAR_COLON) {
                    source.next();
                } else if source.current() == Some(CHAR_COMMA)
                    || source.current() == Some(CHAR_RBRACE)
                {
                    // No colon means implicit null value
                } else {
                    return Err(parse_error(source, ERR_EXPECT_COLON_INLINE_MAPPING));
                }

                key
            }
            Some(CHAR_RBRACE) => {
                // Closing brace without key - this means we're done parsing pairs
                // (handled by the break above after checking CHAR_RBRACE)
                break;
            }
            Some(CHAR_COMMA) => {
                // Comma without key - invalid
                return Err(parse_error(source, "Unexpected comma in flow mapping"));
            }
            _ => {
                // Use collect_flow_scalar to handle multiline plain scalars in flow context
                let collected = collect_flow_scalar(source, |c| {
                    c == CHAR_COLON || c == CHAR_RBRACE || c == CHAR_COMMA
                });
                skip_whitespace_and_comments_validate_tabs(source)?;

                let key_node = {
                    let trimmed = collected.trim();
                    if trimmed.is_empty() {
                        Node::Str(String::new(), QuoteType::Unquoted, BlockStyle::None)
                    } else {
                        parse_scalar(trimmed, directives)
                    }
                };

                // Require a colon for mappings that contain any key-value pairs
                // Cases like {a, b} are sets and handled elsewhere; once we are in the
                // mapping-with-colons path, every entry must have ':' after the key.
                if source.current() == Some(CHAR_COLON) {
                    source.next();
                } else {
                    return Err(parse_error(source, ERR_EXPECT_COLON_INLINE_MAPPING));
                }

                key_node
            }
        };

        skip_whitespace_no_tabs(source)?;

        let value_node = match source.current() {
            // Handle omitted value (key followed immediately by comma or closing brace)
            Some(CHAR_COMMA) | Some(CHAR_RBRACE) => Node::None,
            Some(CHAR_LBRACE) => parse_inline_mapping(source, directives)?,
            Some(CHAR_LBRACKET) => parse_inline_sequence(source, directives)?,
            Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                let raw = parse_quoted_scalar(source)?;
                parse_scalar(raw.trim(), directives)
            }
            Some(_) => {
                // Use collect_flow_scalar for multiline plain scalar values
                let val = collect_flow_scalar(source, |c| {
                    c == CHAR_COMMA || c == CHAR_RBRACE || c == CHAR_HASH
                });
                let trimmed = val.trim();
                // Check if value looks like it contains a mapping (missing comma before next key)
                // Pattern: "value key:" suggests missing comma before "key:"
                if trimmed.contains(':') {
                    // Check if there's a potential mapping key after whitespace
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        // Look for pattern like "1 bar:" where we have value then key
                        for i in 1..parts.len() {
                            if parts[i].ends_with(':')
                                || (i + 1 < parts.len() && parts[i + 1] == ":")
                            {
                                return Err(parse_error(
                                    source,
                                    "Flow mapping missing separating comma between pairs",
                                ));
                            }
                        }
                    }
                }
                parse_scalar(trimmed, directives)
            }
            None => return Err(parse_error(source, ERR_EOF_INLINE_MAPPING)),
        };

        pairs.push((key_node, value_node));

        skip_whitespace_and_comments_validate_tabs(source)?;

        match source.current() {
            Some(CHAR_COMMA) => {
                source.next();
                skip_whitespace_and_comments_validate_tabs(source)?;
                // Check if there's a closing brace after the comma (trailing comma)
                if source.current() == Some(CHAR_RBRACE) {
                    source.next();
                    break;
                }
                // Check for double comma
                if source.current() == Some(CHAR_COMMA) {
                    return Err(parse_error(source, "Flow mapping has consecutive commas"));
                }
                continue;
            }
            Some(CHAR_RBRACE) => {
                source.next();

                // Check for invalid text immediately after closing brace (no space)
                // Valid characters: whitespace, newline, comma, another closing bracket/brace, colon, or comment
                if let Some(c) = source.current() {
                    if !c.is_whitespace()
                        && c != '\n'
                        && c != '\r'
                        && c != ','
                        && c != ']'
                        && c != '}'
                        && c != '#'
                        && c != ':'
                    {
                        // Check if it's an alphanumeric character which would be clearly invalid
                        if c.is_alphanumeric() {
                            return Err(parse_error(
                                source,
                                "Invalid character after flow mapping - expected whitespace or newline",
                            ));
                        }
                    }
                }

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
/// * `directives` - Directive context for tag resolution
///
/// # Returns
///
/// Result containing an Array Node or an error string
pub(crate) fn parse_inline_sequence(
    source: &mut dyn ISource,
    directives: &crate::parser::directives::DirectiveContext,
) -> Result<Node, String> {
    let mut items: Vec<Node> = Vec::new();
    let mut iterations = 0;
    const MAX_ITEMS: usize = 10_000;

    source.next();
    // Validate tabs are not used as indentation after newline in flow context
    skip_whitespace_and_comments_validate_tabs(source)?;

    if source.current() == Some(CHAR_RBRACKET) {
        source.next();
        return Ok(Node::Array(items));
    }

    loop {
        // Prevent infinite loop
        iterations += 1;
        if iterations >= MAX_ITEMS {
            return Err(parse_error(
                source,
                "Flow sequence too large or malformed - possible infinite loop",
            ));
        }

        // Skip whitespace and comments before checking for items
        // Reject tabs used as indentation after newlines in flow context
        skip_whitespace_and_comments_validate_tabs(source)?;

        // Check for closing bracket (handles trailing comma case)
        if source.current() == Some(CHAR_RBRACKET) {
            source.next();
            break;
        }

        // Detect invalid leading comma: if we haven't parsed any items yet and see a comma
        if items.is_empty() && source.current() == Some(CHAR_COMMA) {
            return Err(parse_error(
                source,
                "Flow sequence cannot start with a comma",
            ));
        }

        match source.current() {
            Some(CHAR_LBRACKET) => {
                let nested = parse_inline_sequence(source, directives)?;

                // Check if this flow sequence is being used as an implicit key
                skip_whitespace_and_comments_validate_tabs(source)?;
                if source.current() == Some(CHAR_COLON) {
                    // Flow sequence is a key! Parse the value
                    source.next(); // consume colon
                    skip_whitespace_and_comments_validate_tabs(source)?;

                    let value_node = match source.current() {
                        Some(CHAR_LBRACE) => parse_inline_mapping(source, directives)?,
                        Some(CHAR_LBRACKET) => parse_inline_sequence(source, directives)?,
                        Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                            let raw = parse_quoted_scalar(source)?;
                            parse_scalar(raw.trim(), directives)
                        }
                        Some(CHAR_COMMA) | Some(CHAR_RBRACKET) => Node::None,
                        _ => {
                            let val_str = collect_flow_scalar(source, |c| {
                                c == CHAR_COMMA || c == CHAR_RBRACKET
                            });
                            parse_scalar(val_str.trim(), directives)
                        }
                    };

                    items.push(Node::Mapping(vec![(nested, value_node)]));
                } else {
                    items.push(nested);
                }
            }
            Some(CHAR_LBRACE) => {
                let nested_map = parse_inline_mapping(source, directives)?;

                // Check if this flow mapping is being used as an implicit key
                skip_whitespace_and_comments_validate_tabs(source)?;
                if source.current() == Some(CHAR_COLON) {
                    // Flow mapping is a key! Parse the value
                    source.next(); // consume colon
                    skip_whitespace_and_comments_validate_tabs(source)?;

                    let value_node = match source.current() {
                        Some(CHAR_LBRACE) => parse_inline_mapping(source, directives)?,
                        Some(CHAR_LBRACKET) => parse_inline_sequence(source, directives)?,
                        Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                            let raw = parse_quoted_scalar(source)?;
                            parse_scalar(raw.trim(), directives)
                        }
                        Some(CHAR_COMMA) | Some(CHAR_RBRACKET) => Node::None,
                        _ => {
                            let val_str = collect_flow_scalar(source, |c| {
                                c == CHAR_COMMA || c == CHAR_RBRACKET
                            });
                            parse_scalar(val_str.trim(), directives)
                        }
                    };

                    items.push(Node::Mapping(vec![(nested_map, value_node)]));
                } else {
                    items.push(nested_map);
                }
            }
            // Support anchors and aliases in flow sequences
            Some(CHAR_AMPERSAND) => {
                // Parse anchor, then check if it's followed by an implicit key
                source.next();
                let anchor_name = collect_until(source, |c| {
                    c == CHAR_SPACE
                        || c == CHAR_TAB
                        || c == CHAR_NEWLINE
                        || c == CHAR_CARRIAGE_RETURN
                        || c == CHAR_HASH
                        || c == CHAR_COMMA
                        || c == CHAR_RBRACKET
                });
                if anchor_name.trim().is_empty() {
                    return Err(parse_error(source, "Anchor name cannot be empty"));
                }
                crate::utils::skip_whitespace_and_comments(source);

                // Check for implicit key-value after anchor
                let saved_state = source.save_state();
                let mut found_colon = false;
                let mut brace_depth = 0;
                let mut bracket_depth = 0;
                let mut in_quotes = false;
                let mut quote_char = '\0';

                while let Some(c) = source.current() {
                    match c {
                        CHAR_SINGLE_QUOTE | CHAR_DOUBLE_QUOTE if !in_quotes => {
                            in_quotes = true;
                            quote_char = c;
                        }
                        c if in_quotes && c == quote_char => {
                            in_quotes = false;
                            quote_char = '\0';
                        }
                        CHAR_LBRACE if !in_quotes => brace_depth += 1,
                        CHAR_RBRACE if !in_quotes => {
                            if brace_depth > 0 {
                                brace_depth -= 1;
                            }
                        }
                        CHAR_LBRACKET if !in_quotes => bracket_depth += 1,
                        CHAR_RBRACKET if !in_quotes => {
                            if bracket_depth == 0 {
                                break;
                            }
                            bracket_depth -= 1;
                        }
                        CHAR_COLON if !in_quotes && brace_depth == 0 && bracket_depth == 0 => {
                            found_colon = true;
                            break;
                        }
                        CHAR_COMMA if !in_quotes && brace_depth == 0 && bracket_depth == 0 => break,
                        CHAR_HASH if !in_quotes => break,
                        _ => {}
                    }
                    source.next();
                }

                source.restore_state(saved_state);

                if found_colon {
                    // Parse as anchored implicit mapping
                    let key_node = match source.current() {
                        Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                            let raw = parse_quoted_scalar(source)?;
                            parse_scalar(raw.trim(), directives)
                        }
                        Some(CHAR_LBRACE) => parse_inline_mapping(source, directives)?,
                        Some(CHAR_LBRACKET) => parse_inline_sequence(source, directives)?,
                        _ => {
                            let key_str = collect_flow_scalar(source, |c| {
                                c == CHAR_COLON || c == CHAR_COMMA || c == CHAR_RBRACKET
                            });
                            parse_scalar(key_str.trim(), directives)
                        }
                    };

                    crate::utils::skip_whitespace_and_comments(source);

                    if source.current() != Some(CHAR_COLON) {
                        return Err(parse_error(source, "Expected ':' after implicit key"));
                    }
                    source.next();
                    crate::utils::skip_whitespace_and_comments(source);

                    let value_node = match source.current() {
                        Some(CHAR_LBRACE) => parse_inline_mapping(source, directives)?,
                        Some(CHAR_LBRACKET) => parse_inline_sequence(source, directives)?,
                        Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                            let raw = parse_quoted_scalar(source)?;
                            parse_scalar(raw.trim(), directives)
                        }
                        Some(CHAR_COMMA) | Some(CHAR_RBRACKET) => Node::None,
                        _ => {
                            let val_str = collect_flow_scalar(source, |c| {
                                c == CHAR_COMMA || c == CHAR_RBRACKET || c == CHAR_HASH
                            });
                            let trimmed = val_str.trim();
                            if trimmed.is_empty() {
                                Node::None
                            } else {
                                parse_scalar(trimmed, directives)
                            }
                        }
                    };

                    let mapping = Node::Mapping(vec![(key_node, value_node)]);
                    items.push(Node::Anchored(Box::new(mapping), anchor_name));
                } else {
                    // Parse as anchored regular value
                    let value_node = match source.current() {
                        Some(CHAR_LBRACE) => parse_inline_mapping(source, directives)?,
                        Some(CHAR_LBRACKET) => parse_inline_sequence(source, directives)?,
                        Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                            let raw = parse_quoted_scalar(source)?;
                            parse_scalar(raw.trim(), directives)
                        }
                        _ => {
                            let val = collect_flow_scalar(source, |c| {
                                c == CHAR_COMMA || c == CHAR_RBRACKET || c == CHAR_HASH
                            });
                            let trimmed = val.trim();
                            if trimmed.is_empty() {
                                Node::None
                            } else {
                                parse_scalar(trimmed, directives)
                            }
                        }
                    };
                    items.push(Node::Anchored(Box::new(value_node), anchor_name));
                }
            }
            Some(CHAR_ASTERISK) => {
                let node = parse_value(source, directives)?;
                items.push(node);
            }
            Some(CHAR_COMMA) => {
                // Double comma or missing item between commas
                return Err(parse_error(
                    source,
                    "Flow sequence has missing item (consecutive commas or leading comma)",
                ));
            }
            Some(_) => {
                // Check if this might be an implicit key-value pair (key : value)
                // Save state to potentially backtrack
                let saved_state = source.save_state();
                let mut found_colon = false;
                let mut brace_depth = 0;
                let mut bracket_depth = 0;
                let mut in_quotes = false;
                let mut quote_char = '\0';

                // Scan ahead to see if there's a colon for implicit key
                while let Some(c) = source.current() {
                    match c {
                        CHAR_SINGLE_QUOTE | CHAR_DOUBLE_QUOTE if !in_quotes => {
                            in_quotes = true;
                            quote_char = c;
                        }
                        c if in_quotes && c == quote_char => {
                            // For double quotes, check for escape with backslash (handled by parse_quoted_scalar)
                            // For single quotes, doubled quotes are escapes
                            // For lookahead purposes, just exit quotes (parse_quoted_scalar handles escapes properly)
                            in_quotes = false;
                            quote_char = '\0';
                        }
                        CHAR_LBRACE if !in_quotes => brace_depth += 1,
                        CHAR_RBRACE if !in_quotes => {
                            if brace_depth > 0 {
                                brace_depth -= 1;
                            }
                        }
                        CHAR_LBRACKET if !in_quotes => bracket_depth += 1,
                        CHAR_RBRACKET if !in_quotes => {
                            if bracket_depth == 0 {
                                break; // Reached our closing bracket
                            }
                            bracket_depth -= 1;
                        }
                        CHAR_COLON if !in_quotes && brace_depth == 0 && bracket_depth == 0 => {
                            found_colon = true;
                            break;
                        }
                        CHAR_COMMA if !in_quotes && brace_depth == 0 && bracket_depth == 0 => break,
                        CHAR_HASH if !in_quotes => break, // Comment starts, stop lookahead
                        _ => {}
                    }
                    source.next();
                }

                source.restore_state(saved_state);

                let node = if found_colon {
                    // Parse as implicit key-value pair (single-entry mapping)
                    let key_node = match source.current() {
                        Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                            let raw = parse_quoted_scalar(source)?;
                            parse_scalar(raw.trim(), directives)
                        }
                        Some(CHAR_LBRACE) => parse_inline_mapping(source, directives)?,
                        Some(CHAR_LBRACKET) => parse_inline_sequence(source, directives)?,
                        _ => {
                            let key_str = collect_flow_scalar(source, |c| {
                                c == CHAR_COLON || c == CHAR_COMMA || c == CHAR_RBRACKET
                            });
                            parse_scalar(key_str.trim(), directives)
                        }
                    };

                    skip_whitespace_and_comments_validate_tabs(source)?;

                    if source.current() != Some(CHAR_COLON) {
                        return Err(parse_error(
                            source,
                            "Expected ':' after implicit key in flow sequence",
                        ));
                    }
                    source.next(); // consume colon
                    skip_whitespace_and_comments_validate_tabs(source)?;

                    let value_node = match source.current() {
                        Some(CHAR_LBRACE) => parse_inline_mapping(source, directives)?,
                        Some(CHAR_LBRACKET) => parse_inline_sequence(source, directives)?,
                        Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                            let raw = parse_quoted_scalar(source)?;
                            parse_scalar(raw.trim(), directives)
                        }
                        Some(CHAR_COMMA) | Some(CHAR_RBRACKET) => Node::None,
                        _ => {
                            let val_str = collect_flow_scalar(source, |c| {
                                c == CHAR_COMMA || c == CHAR_RBRACKET
                            });
                            let trimmed = val_str.trim();
                            if trimmed.is_empty() {
                                Node::None
                            } else {
                                parse_scalar(trimmed, directives)
                            }
                        }
                    };

                    // Wrap as single-pair mapping
                    Node::Mapping(vec![(key_node, value_node)])
                } else {
                    // Regular value
                    match source.current() {
                        Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                            let raw = parse_quoted_scalar(source)?;
                            parse_scalar(raw.trim(), directives)
                        }
                        _ => {
                            // Accept leading special characters (e.g., '>folded') as part of the scalar
                            let val = collect_flow_scalar(source, |c| {
                                c == CHAR_COMMA || c == CHAR_RBRACKET || c == CHAR_HASH
                            });
                            let trimmed = val.trim();
                            if trimmed.is_empty() {
                                Node::None
                            } else {
                                // Do not strip leading '>' or other special chars; let parse_scalar handle it
                                if trimmed == "-" {
                                    return Err(parse_error(
                                        source,
                                        "Plain dash in flow sequence is ambiguous",
                                    ));
                                }
                                if trimmed == "---" || trimmed == "..." {
                                    return Err(parse_error(
                                        source,
                                        "Document markers are not allowed in flow collections",
                                    ));
                                }
                                parse_scalar(trimmed, directives)
                            }
                        }
                    }
                };

                if !matches!(node, Node::None) {
                    items.push(node);
                }
            }
            None => return Err(parse_error(source, ERR_EOF_INLINE_SEQUENCE)),
        }

        skip_whitespace_and_comments_validate_tabs(source)?;

        // For now, skip indentation validation in flow context as it's too restrictive
        // TODO: Implement proper multiline flow indentation rules per YAML spec
        // The spec requires flow content to be "more indented" but the exact rules are complex

        match source.current() {
            Some(CHAR_COMMA) => {
                let prev = CHAR_COMMA;
                source.next();
                // Validate comment spacing after comma
                validate_comment_spacing(source, Some(prev))?;
                skip_whitespace_no_tabs(source)?;
                // Check for trailing comma (comma followed by closing bracket)
                if source.current() == Some(CHAR_RBRACKET) {
                    source.next();
                    // Validate comment spacing after closing bracket
                    validate_comment_spacing(source, Some(CHAR_RBRACKET))?;
                    break;
                }
                // Check for double comma
                if source.current() == Some(CHAR_COMMA) {
                    return Err(parse_error(source, "Flow sequence has consecutive commas"));
                }
                continue;
            }
            Some(CHAR_RBRACKET) => {
                let prev = CHAR_RBRACKET;
                source.next();
                // Validate comment spacing after closing bracket
                validate_comment_spacing(source, Some(prev))?;
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
