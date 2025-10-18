//! YAML parser implementation that converts YAML text into Node structures
//! Provides functions for parsing different YAML data types including mappings,
//! sequences, strings, numbers, boolean and null values.

use crate::io::traits::ISource;
use crate::nodes::node::Node::Document;
use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};
use crate::parser::constants::*;
use crate::parser::utils::{
    collect_until, read_line_trimmed_into_string, skip_whitespace_and_comments,
};

// Character constants imported from `crate::parser::constants`

fn unescape_double_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        // handle escape
        match chars.next() {
            Some('n') => out.push(CHAR_NEWLINE),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some(CHAR_DOUBLE_QUOTE) => out.push(CHAR_DOUBLE_QUOTE),
            Some('u') => {
                // \uXXXX
                let mut hex = String::new();
                for _ in 0..4 {
                    if let Some(h) = chars.next() {
                        hex.push(h);
                    } else {
                        break;
                    }
                }
                if let Ok(code) = u32::from_str_radix(&hex, 16) {
                    if let Some(ch) = std::char::from_u32(code) {
                        out.push(ch);
                    }
                }
            }
            Some('U') => {
                // \UXXXXXXXX
                let mut hex = String::new();
                for _ in 0..8 {
                    if let Some(h) = chars.next() {
                        hex.push(h);
                    } else {
                        break;
                    }
                }
                if let Ok(code) = u32::from_str_radix(&hex, 16) {
                    if let Some(ch) = std::char::from_u32(code) {
                        out.push(ch);
                    }
                }
            }
            Some(other) => {
                // Unknown escape, keep the character as-is (e.g., \x -> x)
                out.push(other);
            }
            None => break,
        }
    }
    out
}

// Helper to create richer parse error messages with current character and indent
fn parse_error(source: &mut dyn ISource, msg: &str) -> String {
    let current = match source.current() {
        Some(c) => c.to_string(),
        None => "<EOF>".to_string(),
    };
    format!(
        "{} (current: '{}', indent: {})",
        msg,
        current,
        source.get_current_indent_level()
    )
}

fn skip_whitespace(source: &mut dyn ISource) {
    while let Some(c) = source.current() {
        if source.is_whitespace(c) {
            source.next();
        } else {
            break;
        }
    }
}

// Helper functions moved to `crate::parser::utils`

fn skip_until_newline(source: &mut dyn ISource) {
    while let Some(c) = source.current() {
        if c == CHAR_NEWLINE {
            source.next();
            break;
        }
        source.next();
    }
}

// Helper to determine whether a node (or document) is blank/empty
fn node_is_blank(node: &Node) -> bool {
    match node {
        Node::None => true,
        Node::Array(items) => items.is_empty(),
        // An empty mapping is meaningful ({}), so do not treat it as blank
        Node::Mapping(_pairs) => false,
        Document(nodes) => nodes.iter().all(|n| node_is_blank(n)),
        Node::Str(s, _, _) => s.is_empty(),
        Node::Comment(_) => true,
        _ => false,
    }
}

// Read a quoted flow scalar that may span multiple lines. Returns the raw text including quotes.
fn read_quoted_flow_scalar(source: &mut dyn ISource) -> Result<String, String> {
    let quote = match source.current() {
        Some(c) if c == CHAR_SINGLE_QUOTE || c == CHAR_DOUBLE_QUOTE => c,
        Some(other) => {
            return Err(format!(
                "{}",
                parse_error(source, &format!("Expected quote, found '{}'", other))
            ));
        }
        None => return Err(parse_error(source, "Unexpected EOF while expecting quote")),
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
                        // In single-quoted scalars, doubled single quotes represent a literal single quote
                        if source.current() == Some(CHAR_SINGLE_QUOTE) {
                            out.push(CHAR_SINGLE_QUOTE);
                            source.next();
                            continue;
                        } else {
                            break; // closing quote
                        }
                    } else {
                        // double-quoted: a quote is closing unless escaped by a backslash
                        if prev_was_backslash {
                            // it was escaped, keep going
                            prev_was_backslash = false;
                            continue;
                        } else {
                            break; // closing quote
                        }
                    }
                }

                if quote == CHAR_DOUBLE_QUOTE {
                    if c == '\\' {
                        prev_was_backslash = !prev_was_backslash;
                    } else {
                        prev_was_backslash = false;
                    }
                }
            }
            None => {
                return Err(parse_error(
                    source,
                    "Unterminated quoted scalar in flow context",
                ));
            }
        }
    }

    Ok(out)
}

fn peek_ahead_for_document_start_end(source: &mut dyn ISource, c: char) -> bool {
    if source.current() != Some(c) {
        return false;
    }
    // Save and restore the state; prefer save_state/restore_state for speculative reads
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

fn peek_ahead_for_mapping_key(source: &mut dyn ISource) -> bool {
    let mut found = false;
    let state = source.save_state();

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

    // Restore the original read position
    source.restore_state(state);

    found
}

fn parse_mapping_key(source: &mut dyn ISource) -> Result<(Node, bool), String> {
    // collect until ':' or newline
    let raw = collect_until(source, |c| c == CHAR_COLON || c == CHAR_NEWLINE);

    let mut newline = false;
    source.next(); // Skip ':'
    // Allow no space after ':'; process and skip optional whitespace
    skip_whitespace(source);
    if let Some(c) = source.current() {
        newline = c == CHAR_NEWLINE;
        if newline {
            source.next();
            skip_whitespace(source);
        }
    }

    // parse_scalar expects a &str and returns Node; ensure keys are Str nodes
    match raw.trim() {
        v if v.starts_with(CHAR_HASH) => Ok((
            Node::Str(v.to_string(), QuoteType::Unquoted, BlockStyle::None),
            newline,
        )),
        v => Ok((parse_scalar(v), newline)),
    }
}
fn parse_value(source: &mut dyn ISource) -> Result<Node, String> {
    match source.current() {
        Some(CHAR_LBRACE) => parse_inline_mapping(source),
        Some(CHAR_LBRACKET) => parse_inline_sequence(source),
        Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
            let raw = read_quoted_flow_scalar(source)?;
            let trimmed = raw.trim();
            Ok(parse_scalar(trimmed))
        }
        Some(_) => {
            let value = collect_until(source, |c| c == CHAR_NEWLINE || c == CHAR_HASH);
            let trimmed = value.trim();
            if trimmed == "|" || trimmed == ">" {
                // Minimal block scalar support: collect indented lines following the '|' (literal) or '>' (folded)
                let folded = trimmed == ">";
                // Consume the newline after the block style indicator
                if source.current() == Some(CHAR_NEWLINE) {
                    source.next();
                }
                let mut out = String::new();
                let mut base_indent: Option<usize> = None;
                let mut first = true;
                loop {
                    if source.current().is_none() {
                        break;
                    }
                    // Establish base indent on the first content line by counting leading spaces without consuming
                    if base_indent.is_none() {
                        if source.current() == Some(CHAR_NEWLINE) {
                            break;
                        }
                        let st = source.save_state();
                        let mut count = 0usize;
                        while let Some(' ') = source.current() {
                            count += 1;
                            source.next();
                        }
                        base_indent = Some(count);
                        source.restore_state(st);
                    }
                    let bi = base_indent.unwrap_or(0);
                    // If the current line has less indent than base, stop the block
                    // Count current line's spaces (consuming then restoring)
                    let st_line = source.save_state();
                    let mut cur_indent = 0usize;
                    while let Some(' ') = source.current() {
                        cur_indent += 1;
                        source.next();
                    }
                    source.restore_state(st_line);
                    if cur_indent < bi {
                        break;
                    }

                    // Read current line content
                    let mut line = collect_until(source, |c| c == CHAR_NEWLINE);
                    if folded && !first {
                        // For folded scalars, strip base indentation from later lines
                        let strip = bi.min(line.chars().take_while(|&ch| ch == ' ').count());
                        line.drain(0..strip);
                    }
                    if !first {
                        if folded {
                            out.push(' ');
                        } else {
                            out.push('\n');
                        }
                    } else {
                        first = false;
                    }
                    out.push_str(&line);
                    // Consume a newline if present and continue looping
                    if source.current() == Some(CHAR_NEWLINE) {
                        source.next();
                    } else {
                        break;
                    }
                }
                // Store as Literal block style for consistency with existing expectations
                Ok(Node::Str(out, QuoteType::Unquoted, BlockStyle::Literal))
            } else if !trimmed.is_empty() {
                Ok(parse_scalar(trimmed))
            } else {
                Ok(Node::None)
            }
        }
        None => Ok(Node::None),
    }
}

fn parse_inline_mapping(source: &mut dyn ISource) -> Result<Node, String> {
    // Assumes current char is '{'
    let mut pairs: Vec<(Node, Node)> = Vec::new();
    // consume '{'
    source.next();
    // skip whitespace
    skip_whitespace(source);

    // Handle empty mapping
    if source.current() == Some(CHAR_RBRACE) {
        source.next(); // consume '}'
        return Ok(Node::Mapping(pairs));
    }

    loop {
        // Parse key as Node
        let key_node = {
            let raw = match source.current() {
                Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                    read_quoted_flow_scalar(source)?
                }
                _ => collect_until(source, |c| c == CHAR_COLON || c == CHAR_RBRACE),
            };
            if source.current() != Some(CHAR_COLON) {
                return Err(parse_error(source, ERR_EXPECT_COLON_INLINE_MAPPING));
            }
            // consume ':'
            source.next();
            let trimmed = raw.trim();
            parse_scalar(trimmed)
        };

        // value may start with space
        skip_whitespace(source);

        // Parse value
        let value_node = match source.current() {
            Some(CHAR_LBRACE) => parse_inline_mapping(source)?,
            Some(CHAR_LBRACKET) => parse_inline_sequence(source)?,
            Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                let raw = read_quoted_flow_scalar(source)?;
                parse_scalar(raw.trim())
            }
            Some(_) => {
                // collect until ',' or '}' or '#'
                let val = collect_until(source, |c| {
                    c == CHAR_COMMA || c == CHAR_RBRACE || c == CHAR_HASH
                });
                parse_scalar(val.trim())
            }
            None => return Err(parse_error(source, ERR_EOF_INLINE_MAPPING)),
        };

        pairs.push((key_node, value_node));

        // After value, skip whitespace and optional comment (until the end or before comma/})
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
                    &format!("{}{}", ERR_UNEXPECTED_CHAR_INLINE_MAPPING_PREFIX, c),
                ));
            }
            None => return Err(parse_error(source, ERR_EOF_INLINE_MAPPING)),
        }
    }

    Ok(Node::Mapping(pairs))
}

fn parse_inline_sequence(source: &mut dyn ISource) -> Result<Node, String> {
    // Assumes current char is '['
    let mut items: Vec<Node> = Vec::new();
    // consume '['
    source.next();
    // skip whitespace
    skip_whitespace(source);

    // Handle empty sequence
    if source.current() == Some(CHAR_RBRACKET) {
        source.next(); // consume ']'
        return Ok(Node::Array(items));
    }

    loop {
        // Parse item
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
                // If the item starts with a quote, read a full quoted scalar (may span lines)
                let node = match source.current() {
                    Some(CHAR_SINGLE_QUOTE) | Some(CHAR_DOUBLE_QUOTE) => {
                        let raw = read_quoted_flow_scalar(source)?;
                        parse_scalar(raw.trim())
                    }
                    _ => {
                        // collect until ',' or ']' or '#'
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

        // After the item, skip whitespace and optional comment (until the end of the line)
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
                    &format!("{}{}", ERR_UNEXPECTED_CHAR_INLINE_SEQUENCE_PREFIX, c),
                ));
            }
            None => return Err(parse_error(source, ERR_EOF_INLINE_SEQUENCE)),
        }
    }

    Ok(Node::Array(items))
}

fn parse_comment(source: &mut dyn ISource) -> String {
    source.next(); // Skip the '#' character
    read_line_trimmed_into_string(source)
}

// node_to_map_key is provided by `crate::parser::utils`

fn parse_scalar(value: &str) -> Node {
    // Check if the value is a comment (starts with #)
    match value {
        // Treat leading '#' in a scalar as a plain string. Parser consumes comment lines elsewhere.
        v if v.starts_with(CHAR_HASH) => {
            Node::Str(v.to_string(), QuoteType::Unquoted, BlockStyle::None)
        }
        "null" | "~" => Node::None,
        "true" => Node::Boolean(true),
        "false" => Node::Boolean(false),
        v => {
            if let Ok(i) = v.parse::<i64>() {
                Node::Number(Numeric::Integer(i))
            } else if let Ok(f) = v.parse::<f64>() {
                Node::Number(Numeric::Float(f))
            } else {
                // Determine a quote type based on surrounding characters and strip quotes
                // For double-quoted scalars also unescape common escape sequences

                let (content, qt) = if v.len() >= 2 {
                    let first = v.chars().next().unwrap();
                    let last = v.chars().rev().next().unwrap();
                    if first == CHAR_SINGLE_QUOTE && last == CHAR_SINGLE_QUOTE {
                        // Strip surrounding single quotes
                        let stripped = v[1..v.len() - 1].to_string();
                        (stripped, QuoteType::Single)
                    } else if first == CHAR_DOUBLE_QUOTE && last == CHAR_DOUBLE_QUOTE {
                        // Strip surrounding double quotes and unescape
                        let inner = &v[1..v.len() - 1];
                        (unescape_double_quoted(inner), QuoteType::Double)
                    } else {
                        (v.to_string(), QuoteType::Unquoted)
                    }
                } else {
                    (v.to_string(), QuoteType::Unquoted)
                };
                Node::Str(content, qt, BlockStyle::None)
            }
        }
    }
}

fn parse_sequence(source: &mut dyn ISource, indent_level: usize) -> Result<Node, String> {
    let mut items = Vec::new();
    while let Some(c) = source.current() {
        if source.get_current_indent_level() < indent_level {
            break;
        }

        match c {
            CHAR_HASH => {
                // Consume comment lines inside sequences; do not emit comment nodes
                parse_comment(source);
                continue;
            }
            CHAR_DASH | CHAR_DOT if peek_ahead_for_document_start_end(source, c) => {
                break;
            }
            CHAR_DASH => {
                source.next(); // Skip the dash
                skip_whitespace(source);
                if source.current() == Some(CHAR_NEWLINE) {
                    source.next();
                    skip_whitespace(source);
                }

                if let Some(next_c) = source.current() {
                    match next_c {
                        CHAR_DASH => {
                            // Check for a nested sequence
                            let nested_indent = source.get_current_indent_level();
                            items.push(parse_document_contents(source, nested_indent)?);
                            continue;
                        }
                        CHAR_LBRACKET | CHAR_LBRACE => {
                            // Inline collection (flow sequence or mapping) — parse directly
                            items.push(parse_value(source)?);
                            continue;
                        }
                        _ => {
                            if peek_ahead_for_mapping_key(source) {
                                let nested_indent = source.get_current_indent_level();
                                items.push(parse_document_contents(source, nested_indent)?);
                                continue;
                            }
                            // Parse scalar or plain value
                            else {
                                items.push(parse_value(source)?);
                            }
                        }
                    }
                }
            }
            _ if !c.is_whitespace() => {
                return Err(format!(
                    "Expected sequence item starting with CHAR_DASH, got '{}'",
                    c
                ));
            }
            _ => (),
        }

        skip_until_newline(source);
        skip_whitespace(source);
    }
    Ok(Node::Array(items))
}

fn parse_mapping(source: &mut dyn ISource, indent_level: usize) -> Result<Node, String> {
    let mut pairs: Vec<(Node, Node)> = Vec::new();
    while let Some(c) = source.current() {
        match c {
            CHAR_DASH | CHAR_DOT if peek_ahead_for_document_start_end(source, c) => {
                break;
            }
            CHAR_HASH => {
                parse_comment(source);
            }
            c if c.is_alphanumeric() => {
                if source.get_current_indent_level() < indent_level {
                    break;
                }

                let (key_node, newline) = parse_mapping_key(source)?;

                let next_indent = source.get_current_indent_level();
                if next_indent > indent_level && newline {
                    pairs.push((key_node, parse_document_contents(source, next_indent)?));
                    continue;
                } else {
                    pairs.push((key_node, parse_value(source)?));
                }
            }
            c if c.is_whitespace() => {
                source.next();
                continue;
            }
            _ => break,
        }
        skip_until_newline(source);
        skip_whitespace(source);
    }
    // Sort pairs by key for deterministic output
    Ok(Node::Mapping(pairs))
}

pub fn parse_document_contents(
    source: &mut dyn ISource,
    indent_level: usize,
) -> Result<Node, String> {
    match source.current() {
        Some(CHAR_DASH) => {
            let indent_level = source.get_current_indent_level();
            Ok(parse_sequence(source, indent_level)?)
        }
        Some(CHAR_HASH) => {
            // Consume the comment line and continue parsing the next content
            parse_comment(source);
            skip_whitespace(source);
            parse_document_contents(source, indent_level)
        }
        Some(CHAR_LBRACE) => Ok(parse_inline_mapping(source)?),
        Some(CHAR_LBRACKET) => Ok(parse_inline_sequence(source)?),
        Some(CHAR_QUESTION) => {
            // Minimal explicit pair support for a pattern? [ ... ] then : value
            source.next();
            skip_whitespace(source);
            // Parse key (support only a flow sequence or plain scalar until EOL)
            let key_node = match source.current() {
                Some(CHAR_LBRACKET) => parse_inline_sequence(source)?,
                Some(_) => Node::Str(
                    read_line_trimmed_into_string(source),
                    QuoteType::Unquoted,
                    BlockStyle::None,
                ),
                None => Node::Str(String::new(), QuoteType::Unquoted, BlockStyle::None),
            };
            if source.current() == Some(CHAR_NEWLINE) {
                source.next();
            }
            loop {
                skip_whitespace(source);
                if source.current() == Some(CHAR_COLON) {
                    break;
                }
                if source.current().is_none() {
                    break;
                }
                skip_until_newline(source);
                if source.current().is_none() {
                    break;
                }
            }
            if source.current() == Some(CHAR_COLON) {
                source.next();
            }
            skip_whitespace(source);
            let value_node = match source.current() {
                Some(CHAR_LBRACKET) => parse_inline_sequence(source)?,
                Some(CHAR_LBRACE) => parse_inline_mapping(source)?,
                Some(CHAR_DASH) => {
                    let nested_indent = source.get_current_indent_level();
                    parse_sequence(source, nested_indent)?
                }
                Some(_) => parse_value(source)?,
                None => {
                    return Err(parse_error(
                        source,
                        "Unexpected end of input while parsing explicit pair value",
                    ));
                }
            };
            // Build a mapping where the key is a Node (preserving quote metadata)
            let mut pairs: Vec<(Node, Node)> = Vec::new();
            pairs.push((key_node, value_node));
            Ok(Node::Mapping(pairs))
        }
        Some(c) if c.is_alphanumeric() => Ok(parse_mapping(source, indent_level)?),
        Some(c) if c.is_whitespace() => {
            source.next();
            Ok(parse_document_contents(source, indent_level)?)
        }
        Some(CHAR_NUL) => {
            // Treat NUL as ignorable whitespace/end padding
            source.next();
            Ok(parse_document_contents(source, indent_level)?)
        }
        Some(CHAR_LESS)
        | Some(CHAR_GREATER)
        | Some(CHAR_DOUBLE_QUOTE)
        | Some(CHAR_SINGLE_QUOTE) => {
            // Allow certain scalar format strings to start with special characters
            Ok(parse_value(source)?)
        }
        Some(c) => Err(parse_error(source, &format!("Unexpected character: {}", c))),
        None => Ok(Node::None),
    }
}
pub fn parse_document(source: &mut dyn ISource, indent_level: usize) -> Result<Node, String> {
    skip_whitespace(source);

    let mut document_nodes = Vec::new();

    while let Some(c) = source.current() {
        match c {
            CHAR_DASH | CHAR_DOT if peek_ahead_for_document_start_end(source, c) => {
                skip_until_newline(source);
                skip_whitespace(source);
                break;
            }
            CHAR_HASH => {
                // Skip top-level comment lines
                parse_comment(source);
                skip_whitespace(source);
                continue;
            }
            _ => {
                document_nodes.push(parse_document_contents(source, indent_level)?);
            }
        }
    }

    Ok(Document(document_nodes))
}
pub fn parse(source: &mut dyn ISource) -> Result<Node, String> {
    // Use module-level helper `node_is_blank`
    let mut docs: Vec<Node> = Vec::new();
    if peek_ahead_for_document_start_end(source, CHAR_DASH) {
        skip_until_newline(source);
        skip_whitespace(source);
    }
    while source.more() {
        let document = parse_document(source, 0);
        match document {
            Ok(doc) => {
                // Only add non-empty Document nodes. This strips out
                // comment-only documents that would otherwise appear
                // between explicit document markers.
                let is_blank_doc = match &doc {
                    Document(nodes) => nodes.iter().all(|n| node_is_blank(n)),
                    _ => false,
                };
                if !is_blank_doc {
                    docs.push(doc.into())
                }
            }
            Err(err) => {
                return Err(err);
            }
        };
    }
    if docs.is_empty() {
        docs.push(Document(Vec::new()))
    }
    Ok(Node::Documents(docs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::io::sources::file::File as FileSource;
    use std::collections::HashMap;
    use std::fs;

    // NOTE: Mappings preserve insertion order in the parser. The tests below therefore
    // explicitly construct expected `Node::Mapping(Vec<(Node, Node)>)` values in the
    // source order (instead of building expectations from a `HashMap`) to avoid
    // nondeterministic iteration order causing test failures.

    // (Removed) helper: map_from_hashmap_inline

    fn get_json_file_paths(directory: &str) -> Vec<String> {
        let mut paths = Vec::new();
        if let Ok(entries) = fs::read_dir(directory) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                        if let Some(path_str) = path.to_str() {
                            paths.push(path_str.to_string());
                        }
                    }
                }
            }
        }
        paths
    }

    #[test]
    fn test_parse_yaml_files() {
        let files_dir = "../files";
        let json_files = get_json_file_paths(files_dir);
        for file_path in json_files {
            match FileSource::new(&file_path.to_string()) {
                Ok(mut source) => {
                    let result = parse(&mut source);
                    assert!(
                        result.is_ok(),
                        "Failed to parse {}: {:?}",
                        file_path,
                        result.err()
                    );
                }
                Err(e) => panic!("Failed to open {}: {}", file_path, e),
            }
        }
    }

    #[test]
    fn test_parse_scalar() {
        assert_eq!(parse_scalar("null"), Node::None);
        assert_eq!(parse_scalar("~"), Node::None);
        assert_eq!(parse_scalar("true"), Node::Boolean(true));
        assert_eq!(parse_scalar("false"), Node::Boolean(false));
        assert_eq!(parse_scalar("42"), Node::Number(Numeric::Integer(42)));
        assert_eq!(parse_scalar("3.14"), Node::Number(Numeric::Float(3.14)));
        assert_eq!(
            parse_scalar("hello"),
            Node::Str("hello".to_string(), QuoteType::Unquoted, BlockStyle::None)
        );
        assert_eq!(
            parse_scalar("#comment"),
            Node::Str(
                "#comment".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None
            )
        );
    }

    #[test]
    fn test_parse_sequence() {
        let mut source = Buffer::new(b"- 1\n- 2\n- 3");
        let result = parse(&mut source).unwrap();
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Number(Numeric::Integer(2)),
                Node::Number(Numeric::Integer(3))
            ])])])
        );
    }

    #[test]
    fn test_parse_sequence_with_comments() {
        let mut source = Buffer::new(b"- 1\n# Comment 1\n- 2\n# Comment 2");
        let result = parse(&mut source).unwrap();
        // Comments are stripped; a sequence should contain only the items
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Number(Numeric::Integer(2))
            ])])])
        );
    }

    #[test]
    fn test_parse_mapping() {
        let mut source = Buffer::new(b"key1: value1\nkey2: 42");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![
            (
                Node::Str("key1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            ),
            (
                Node::Str("key2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(42)),
            ),
        ])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_empty() {
        let mut source = Buffer::new(b"");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![Document(vec![])]));
    }

    #[test]
    fn test_parse_invalid_char() {
        let mut source = Buffer::new(b"@invalid");
        let result = parse(&mut source);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Unexpected character: @"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_parse_comment_only() {
        let mut source = Buffer::new(b"# Just a comment");
        let result = parse(&mut source).unwrap();
        // The parser strips comments; comment-only content yields an empty document
        assert_eq!(result, Node::Documents(vec![Document(vec![])]));
    }

    #[test]
    fn test_parse_multi_document() {
        let mut source =
            Buffer::new(b"key1: value1\n---\nkey2: value2\n---\nkey3: value3\nkey4: value4\n");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![
            Document(vec![Node::Mapping(vec![(
                Node::Str("key1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            )])]),
            Document(vec![Node::Mapping(vec![(
                Node::Str("key2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value2".to_string(), QuoteType::Unquoted, BlockStyle::None),
            )])]),
            Document(vec![Node::Mapping(vec![
                (
                    Node::Str("key3".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("value3".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
                (
                    Node::Str("key4".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("value4".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
            ])]),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_header_comments() {
        let mut source = Buffer::new(
            b"# Header comment 1\n# Header comment 2\n# Header comment 3\nkey: value\n",
        );
        let result = parse(&mut source).unwrap();
        // Header comments are stripped; only the mapping should remain
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![{
                let mut map = HashMap::new();
                map.insert(
                    "key".to_string(),
                    Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
                );
                let mut pairs = Vec::new();
                for (k, v) in map.into_iter() {
                    let value = match v {
                        Node::Mapping(p) => Node::Mapping(p),
                        other => other,
                    };
                    pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                }
                Node::Mapping(pairs)
            }])])
        );
    }

    #[test]
    fn test_parse_nested_sequence() {
        let mut source = Buffer::new(b"- item1\n- - nested1\n  - nested2\n- item2");
        let result = parse(&mut source).unwrap();
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![Node::Array(vec![
                Node::Str("item1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Array(vec![
                    Node::Str("nested1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("nested2".to_string(), QuoteType::Unquoted, BlockStyle::None)
                ]),
                Node::Str("item2".to_string(), QuoteType::Unquoted, BlockStyle::None)
            ])])])
        );
    }

    #[test]
    fn test_parse_mapping_with_comments() {
        let mut source = Buffer::new(b"key1: value1\n# Comment 1\nkey2: 42\n# Comment 2");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![
            (
                Node::Str("key1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            ),
            (
                Node::Str("key2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(42)),
            ),
        ])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_nested_mapping() {
        let mut source = Buffer::new(b"outer:\n  inner1: value1\n  inner2: value2");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("outer".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Mapping(vec![
                (
                    Node::Str("inner1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("value1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
                (
                    Node::Str("inner2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("value2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
            ]),
        )])])]);

        assert_eq!(result, expected);
    }
    #[test]
    fn test_parse_nested_mapping_with_key_after_nested() {
        let mut source =
            Buffer::new(b"outer1:\n  inner1: value1\n  inner2: value2\nouter2: value3");
        let result = parse(&mut source).unwrap();

        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![
            (
                Node::Str("outer1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Mapping(vec![
                    (
                        Node::Str("inner1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        Node::Str("value1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    ),
                    (
                        Node::Str("inner2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        Node::Str("value2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    ),
                ]),
            ),
            (
                Node::Str("outer2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value3".to_string(), QuoteType::Unquoted, BlockStyle::None),
            ),
        ])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_mapping_with_nested_sequence() {
        let mut source = Buffer::new(b"key1:\n  - item1\n  - item2\nkey2: value2");
        let result = parse(&mut source).unwrap();

        let sequence = Node::Array(vec![
            Node::Str("item1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("item2".to_string(), QuoteType::Unquoted, BlockStyle::None),
        ]);

        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![
            (
                Node::Str("key1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                sequence,
            ),
            (
                Node::Str("key2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("value2".to_string(), QuoteType::Unquoted, BlockStyle::None),
            ),
        ])])]);

        assert_eq!(result, expected);
    }
    #[test]
    fn test_parse_mapping_with_nested_sequence_and_comments() {
        let mut source =
            Buffer::new(b"key1:\n  - item1\n  - item2\n# Comment 1\nkey2: value2\n# Comment 2");
        let result = parse(&mut source).unwrap();
        let sequence = Node::Array(vec![
            Node::Str("item1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("item2".to_string(), QuoteType::Unquoted, BlockStyle::None),
        ]);
        let expected = Node::Documents(vec![Document(vec![
            // Node::Comment("Comment 1".to_string()),
            Node::Mapping(vec![
                (
                    Node::Str("key1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    sequence,
                ),
                (
                    Node::Str("key2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("value2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
            ]),
            // Node::Comment("Comment 2".to_string())
        ])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_sequence_with_nested_comments() {
        let mut source =
            Buffer::new(b"- item1\n# Comment between items\n- item2\n# Final comment\n- item3");
        let result = parse(&mut source).unwrap();
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![Node::Array(vec![
                Node::Str("item1".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("item2".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("item3".to_string(), QuoteType::Unquoted, BlockStyle::None)
            ])])])
        );
    }

    #[test]
    fn test_parse_document_end_marker() {
        let mut source = Buffer::new(b"key: value\n---");
        let result = parse(&mut source).unwrap();
        let mut expected = HashMap::new();
        expected.insert(
            "key".to_string(),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        );
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![{
                let mut pairs = Vec::new();
                for (k, v) in expected.into_iter() {
                    let value = match v {
                        Node::Mapping(p) => Node::Mapping(p),
                        other => other,
                    };
                    pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                }
                Node::Mapping(pairs)
            }])])
        );
    }

    #[test]
    fn test_parse_document_end_marker_with_trailing_content() {
        let mut source = Buffer::new(b"key: value\n---\nother: 123");
        let result = parse(&mut source).unwrap();
        let mut doc1 = HashMap::new();
        doc1.insert(
            "key".to_string(),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        );
        let mut doc2 = HashMap::new();
        doc2.insert("other".to_string(), Node::Number(Numeric::Integer(123)));
        assert_eq!(
            result,
            Node::Documents(vec![
                Document(vec![{
                    let mut pairs = Vec::new();
                    for (k, v) in doc1.into_iter() {
                        let value = match v {
                            Node::Mapping(p) => Node::Mapping(p),
                            other => other,
                        };
                        pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                    }
                    Node::Mapping(pairs)
                }]),
                Document(vec![{
                    let mut pairs = Vec::new();
                    for (k, v) in doc2.into_iter() {
                        let value = match v {
                            Node::Mapping(p) => Node::Mapping(p),
                            other => other,
                        };
                        pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                    }
                    Node::Mapping(pairs)
                }])
            ])
        );
    }

    #[test]
    fn test_parse_document_end_marker_with_comments() {
        let mut source = Buffer::new(b"# Comment before\nkey: value\n---\n# After doc\nother: 1");
        let result = parse(&mut source).unwrap();
        let mut doc1 = HashMap::new();
        doc1.insert(
            "key".to_string(),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        );
        let mut doc2 = HashMap::new();
        doc2.insert("other".to_string(), Node::Number(Numeric::Integer(1)));
        assert_eq!(
            result,
            Node::Documents(vec![
                Document(vec![{
                    let mut pairs = Vec::new();
                    for (k, v) in doc1.into_iter() {
                        let value = match v {
                            Node::Mapping(p) => Node::Mapping(p),
                            other => other,
                        };
                        pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                    }
                    Node::Mapping(pairs)
                }]),
                Document(vec![{
                    let mut pairs = Vec::new();
                    for (k, v) in doc2.into_iter() {
                        let value = match v {
                            Node::Mapping(p) => Node::Mapping(p),
                            other => other,
                        };
                        pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                    }
                    Node::Mapping(pairs)
                }])
            ])
        );
    }

    #[test]
    fn test_parse_document_end_marker_only() {
        let mut source = Buffer::new(b"---");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![Document(vec![])]));
    }

    #[test]
    fn test_parse_multiple_document_end_markers() {
        let mut source = Buffer::new(b"key: value\n---\n---\nother: 1");
        let result = parse(&mut source).unwrap();
        let mut doc1 = HashMap::new();
        doc1.insert(
            "key".to_string(),
            Node::Str("value".to_string(), QuoteType::Unquoted, BlockStyle::None),
        );
        let mut doc3 = HashMap::new();
        doc3.insert("other".to_string(), Node::Number(Numeric::Integer(1)));
        assert_eq!(
            result,
            Node::Documents(vec![
                Document(vec![{
                    let mut pairs = Vec::new();
                    for (k, v) in doc1.into_iter() {
                        let value = match v {
                            Node::Mapping(p) => Node::Mapping(p),
                            other => other,
                        };
                        pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                    }
                    Node::Mapping(pairs)
                }]),
                Document(vec![{
                    let mut pairs = Vec::new();
                    for (k, v) in doc3.into_iter() {
                        let value = match v {
                            Node::Mapping(p) => Node::Mapping(p),
                            other => other,
                        };
                        pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                    }

                    Node::Mapping(pairs)
                }])
            ])
        );
    }
    #[test]
    fn test_parse_nested_mapping_within_sequence() {
        let mut source =
            Buffer::new(b"people:\n  - name: John\n    likes:\n      - apples\n      - bananas\n");
        let result = parse(&mut source).unwrap();

        // Expected: people -> [ { name: "John", likes: ["apples", "bananas"] } ]
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("people".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Array(vec![Node::Mapping(vec![
                (
                    Node::Str("name".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("John".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
                (
                    Node::Str("likes".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Array(vec![
                        Node::Str("apples".to_string(), QuoteType::Unquoted, BlockStyle::None),
                        Node::Str("bananas".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    ]),
                ),
            ])]),
        )])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_sequence_of_mappings() {
        let yaml = b"-\n  name: Mark Joseph\n  hr: 87\n  avg: 0.278\n-\n  name: James Stephen\n  hr: 63\n  avg: 0.288\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source).unwrap();

        let mut mark_map = HashMap::new();
        mark_map.insert(
            "name".to_string(),
            Node::Str(
                "Mark Joseph".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ),
        );
        mark_map.insert("hr".to_string(), Node::Number(Numeric::Integer(87)));
        mark_map.insert("avg".to_string(), Node::Number(Numeric::Float(0.278)));

        let mut james_map = HashMap::new();
        james_map.insert(
            "name".to_string(),
            Node::Str(
                "James Stephen".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ),
        );
        james_map.insert("hr".to_string(), Node::Number(Numeric::Integer(63)));
        james_map.insert("avg".to_string(), Node::Number(Numeric::Float(0.288)));

        let expected = Node::Documents(vec![Document(vec![Node::Array(vec![
            Node::Mapping(vec![
                (
                    Node::Str("name".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str(
                        "Mark Joseph".to_string(),
                        QuoteType::Unquoted,
                        BlockStyle::None,
                    ),
                ),
                (
                    Node::Str("hr".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Number(Numeric::Integer(87)),
                ),
                (
                    Node::Str("avg".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Number(Numeric::Float(0.278)),
                ),
            ]),
            Node::Mapping(vec![
                (
                    Node::Str("name".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str(
                        "James Stephen".to_string(),
                        QuoteType::Unquoted,
                        BlockStyle::None,
                    ),
                ),
                (
                    Node::Str("hr".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Number(Numeric::Integer(63)),
                ),
                (
                    Node::Str("avg".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Number(Numeric::Float(0.288)),
                ),
            ]),
        ])])]);

        assert_eq!(result, expected);
    }
    #[test]
    fn test_peek_ahead_for_mapping_key_basic() {
        let mut source = Buffer::new(b"key: value");
        assert_eq!(source.get_current_indent_level(), 0);
        assert!(peek_ahead_for_mapping_key(&mut source));
        assert_eq!(source.get_current_indent_level(), 0);
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_no_colon() {
        let mut source = Buffer::new(b"key value");
        assert!(!peek_ahead_for_mapping_key(&mut source));
        assert_eq!(source.get_current_indent_level(), 0);
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_colon_after_newline() {
        let mut source = Buffer::new(b"key\n: value");
        assert!(!peek_ahead_for_mapping_key(&mut source));
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_spaces_before_colon() {
        let mut source = Buffer::new(b"key   : value");
        assert!(peek_ahead_for_mapping_key(&mut source));
    }

    #[test]
    fn test_peek_ahead_for_mapping_key_empty() {
        let mut source = Buffer::new(b"");
        assert!(!peek_ahead_for_mapping_key(&mut source));
    }

    #[test]
    fn test_parse_inline_mapping_top_level() {
        let mut source = Buffer::new(b"{a: 1, b: 2}");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![
            (
                Node::Str("a".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(1)),
            ),
            (
                Node::Str("b".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Number(Numeric::Integer(2)),
            ),
        ])])]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_inline_mapping_empty() {
        let mut source = Buffer::new(b"{}");
        let result = parse(&mut source).unwrap();
        let map: HashMap<String, Node> = HashMap::new();
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![{
                let mut pairs = Vec::new();
                for (k, v) in map.into_iter() {
                    let value = match v {
                        Node::Mapping(p) => Node::Mapping(p),
                        other => other,
                    };
                    pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                }

                Node::Mapping(pairs)
            }])])
        );
    }

    #[test]
    fn test_parse_inline_mapping_as_value() {
        let mut source = Buffer::new(b"parent: {a: 1, b: test}");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("parent".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Mapping(vec![
                (
                    Node::Str("a".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Number(Numeric::Integer(1)),
                ),
                (
                    Node::Str("b".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
            ]),
        )])])]);

        assert_eq!(result, expected);
    }

    // New tests for block and flow scalar format strings
    #[test]
    fn test_block_scalar_like_string_same_line() {
        // '>' at start of value should be treated as a plain string on the same line
        let mut source = Buffer::new(b"key: > hello world");
        let result = parse(&mut source).unwrap();
        let mut expected = HashMap::new();
        expected.insert(
            "key".to_string(),
            Node::Str(
                "> hello world".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ),
        );
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![{
                let mut pairs = Vec::new();
                for (k, v) in expected.into_iter() {
                    let value = match v {
                        Node::Mapping(p) => Node::Mapping(p),
                        other => other,
                    };
                    pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                }

                Node::Mapping(pairs)
            }])])
        );
    }

    #[test]
    fn test_block_scalar_like_string_next_line() {
        // When the value line starts with '>' on the next indented line, treat it as plain string too
        let mut source = Buffer::new(b"key:\n  > multi line");
        let result = parse(&mut source).unwrap();
        let mut expected = HashMap::new();
        expected.insert(
            "key".to_string(),
            Node::Str(
                "> multi line".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            ),
        );
        assert_eq!(
            result,
            Node::Documents(vec![Document(vec![{
                let mut pairs = Vec::new();
                for (k, v) in expected.into_iter() {
                    let value = match v {
                        Node::Mapping(p) => Node::Mapping(p),
                        other => other,
                    };
                    pairs.push((Node::Str(k, QuoteType::Unquoted, BlockStyle::None), value));
                }

                Node::Mapping(pairs)
            }])])
        );
    }

    #[test]
    fn test_flow_sequence_with_special_leading_chars_and_quotes() {
        // In a flow sequence, items that start with special chars or quotes are kept as-is (no unquoting)
        let mut source = Buffer::new(b"[<tag, 'quoted', \"double\", >folded]");
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Array(vec![
            Node::Str("<tag".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("quoted".to_string(), QuoteType::Single, BlockStyle::None),
            Node::Str("double".to_string(), QuoteType::Double, BlockStyle::None),
            Node::Str(">folded".to_string(), QuoteType::Unquoted, BlockStyle::None),
        ])])]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_flow_multiline_double_quoted_in_sequence() {
        let yaml = b"[\n\"line1\nline2\", 2\n]";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Array(vec![
            Node::Str(
                "line1\nline2".to_string(),
                QuoteType::Double,
                BlockStyle::None,
            ),
            Node::Number(Numeric::Integer(2)),
        ])])]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_flow_multiline_single_quoted_mapping_value() {
        let yaml = b"{a: 'hello\nworld'}";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("a".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str(
                "hello\nworld".to_string(),
                QuoteType::Single,
                BlockStyle::None,
            ),
        )])])]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_flow_multiline_quoted_key_in_inline_mapping() {
        let yaml = b"{\"multi\nline\": 1}";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str(
                "multi\nline".to_string(),
                QuoteType::Double,
                BlockStyle::None,
            ),
            Node::Number(Numeric::Integer(1)),
        )])])]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_empty_document_end_marker() {
        let mut source = Buffer::new(b"...");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Documents(vec![Document(vec![])]));
    }

    #[test]
    fn test_parse_literal_block_literal_scalar_with_indent() {
        let yaml = b"---\nstring1: |\n  Line1\n  line2\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source).unwrap();
        let expected = Node::Documents(vec![Document(vec![Node::Mapping(vec![(
            Node::Str("string1".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str(
                "  Line1\n  line2".to_string(),
                QuoteType::Unquoted,
                BlockStyle::Literal,
            ),
        )])])]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_literal_block_folded_scalar_with_indent() {
        use crate::io::destinations::buffer::Buffer as DestBuffer;
        use crate::stringify;

        let yaml = b"---\nstring1: >\n  Line1\n  line2\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source).unwrap();

        // Verify stringifier output of the parsed node
        let mut dest = DestBuffer::new();
        stringify(&result, &mut dest).unwrap();
        let out = dest.to_string();
        // Expect a single-line folded value; note there is one space after ':' from mapping formatting
        // plus two spaces that are part of the scalar itself, totaling three spaces before 'Line1'.
        assert_eq!(out, "---\nstring1: |\n  Line1 line2\n...\n");
    }
}
