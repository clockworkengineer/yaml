//! Module: utils/mod.rs

/// Performance measurement and optimization utilities
pub mod performance;

/// String interning for memory optimization
#[cfg(feature = "alloc")]
pub mod string_interner;

/// Performance optimization utilities (lazy evaluation, capacity hints, zero-copy)
#[cfg(feature = "alloc")]
pub mod optimization;

pub mod escape;
/// Streaming and iterator support for efficient YAML processing
#[cfg(feature = "alloc")]
pub mod streaming;

/// Anchor-related helpers for DRY refactor
pub mod anchors_helpers;
pub mod anchors_helpers2;

use crate::constants::{
    CHAR_CARRIAGE_RETURN, CHAR_HASH, CHAR_NEWLINE, CHAR_SPACE, CHAR_TAB,
};
use crate::io::traits::ISource;
use crate::{Node, Numeric};

/// Returns true if the character is a line terminator ("\n" or "\r").
#[inline]
pub fn is_line_terminator(c: char) -> bool {
    c == CHAR_NEWLINE || c == CHAR_CARRIAGE_RETURN
}

/// Returns true if the character is horizontal whitespace (space or tab).
#[inline]
pub fn is_horizontal_space(c: char) -> bool {
    c == CHAR_SPACE || c == CHAR_TAB
}

/// Returns true if the character starts a comment ("#").
#[inline]
pub fn is_comment_start(c: char) -> bool {
    c == CHAR_HASH
}

/// Collects characters from the source until the stop predicate returns true.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
/// * `stop_pred` - A closure that takes a character and returns true when collection should stop
///
/// # Returns
///
/// A String containing all collected characters before the stop condition
///
/// # Safety
///
/// Has a maximum iteration limit of 100,000 characters to prevent infinite loops
pub fn collect_until<F>(source: &mut dyn ISource, mut stop_pred: F) -> String
where
    F: FnMut(char) -> bool,
{
    let mut out = String::new();
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 100_000;

    while let Some(c) = source.current() {
        if stop_pred(c) {
            break;
        }
        out.push(c);
        source.next();

        iterations += 1;
        if iterations >= MAX_ITERATIONS {
            // Prevent infinite loop - return what we have so far
            break;
        }
    }
    out
}

/// Skips whitespace characters and comment lines in the source.
///
/// Continuously advances through the source, skipping whitespace and comment lines
/// (lines starting with #) until non-whitespace, non-comment content is encountered.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
///
/// # Safety
///
/// Has a maximum iteration limit of 100,000 characters to prevent infinite loops
pub fn skip_whitespace_and_comments(source: &mut dyn ISource) {
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 100_000;

    loop {
        while let Some(c) = source.current() {
            if source.is_whitespace(c) || is_line_terminator(c) {
                source.next();
                iterations += 1;
                if iterations >= MAX_ITERATIONS {
                    return; // Prevent infinite loop
                }
            } else {
                break;
            }
        }
        if source.current().map_or(false, is_comment_start) {
            while let Some(c) = source.current() {
                if is_line_terminator(c) {
                    source.next(); // consume the newline after comment
                    break;
                }
                source.next();
                iterations += 1;
                if iterations >= MAX_ITERATIONS {
                    return; // Prevent infinite loop
                }
            }

            continue;
        }
        break;
    }
}

/// Skips whitespace and comments, validating that tabs are not used as indentation
/// after newlines. Per YAML 1.2 spec, tabs cannot be used for indentation.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
///
/// # Returns
///
/// `Ok(())` if successful, `Err(String)` if tabs found as indentation
pub fn skip_whitespace_and_comments_validate_tabs(source: &mut dyn ISource) -> Result<(), String> {
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 100_000;
    let mut after_newline = false;

    loop {
        while let Some(c) = source.current() {
            if is_line_terminator(c) {
                source.next();
                after_newline = true;
                iterations += 1;
                if iterations >= MAX_ITERATIONS {
                    return Ok(()); // Prevent infinite loop
                }
            } else if c == CHAR_TAB && after_newline {
                // Tab after newline is indentation - forbidden in YAML 1.2
                return Err(format!(
                    "Tabs cannot be used for indentation in YAML (current: '{}', indent: {})",
                    c,
                    source.get_current_indent_level()
                ));
            } else if source.is_whitespace(c) {
                source.next();
                iterations += 1;
                if iterations >= MAX_ITERATIONS {
                    return Ok(()); // Prevent infinite loop
                }
            } else {
                // Non-whitespace found
                after_newline = false;
                break;
            }
        }
        if source.current().map_or(false, is_comment_start) {
            while let Some(c) = source.current() {
                if is_line_terminator(c) {
                    source.next(); // consume the newline after comment
                    after_newline = true;
                    break;
                }
                source.next();
                iterations += 1;
                if iterations >= MAX_ITERATIONS {
                    return Ok(()); // Prevent infinite loop
                }
            }

            continue;
        }
        break;
    }
    Ok(())
}

/// Skips characters in the source until a newline character is encountered.
///
/// Advances through the source character by character until it finds a newline
/// character ('\n'), which is also consumed.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
pub fn skip_until_newline(source: &mut dyn ISource) {
    while let Some(c) = source.current() {
        if c == CHAR_NEWLINE {
            source.next();
            break;
        }
        source.next();
    }
}

/// Reads a line from the source and returns it as a trimmed string, excluding comments.
///
/// Collects characters until a newline is encountered. If a hash (#) character is found,
/// everything from the hash to the end of the line is treated as a comment and excluded.
/// The resulting string is trimmed of leading and trailing whitespace.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
///
/// # Returns
///
/// A trimmed String containing the line content without comments
pub fn read_line_trimmed_into_string(source: &mut dyn ISource) -> String {
    let s = collect_until(source, |c| c == CHAR_NEWLINE);

    if let Some(pos) = s.find(CHAR_HASH) {
        return s[..pos].trim().to_string();
    }
    s.trim().to_string()
}

/// Converts a Node to its inline string representation.
///
/// Transforms various Node types into compact string representations suitable for
/// inline display. Arrays and mappings are formatted in compact JSON-like syntax.
///
/// # Arguments
///
/// * `node` - A reference to the Node to convert
///
/// # Returns
///
/// A String containing the inline representation of the node
#[allow(dead_code)]
pub fn node_to_inline_string(node: &Node) -> String {
    match node {
        Node::Str(s, _, _) => s.clone(),
        Node::Number(Numeric::Integer(i)) => i.to_string(),
        Node::Number(Numeric::Float(f)) => f.to_string(),
        Node::Boolean(b) => b.to_string(),
        Node::Array(items) => {
            let parts: Vec<String> = items.iter().map(node_to_inline_string).collect();
            format!("[{}]", parts.join(", "))
        }
        Node::Mapping(pairs) => {
            let parts: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("{}: {}", node_to_inline_string(k), node_to_inline_string(v)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        _ => format!("{node:?}"),
    }
}

/// Unescapes a double-quoted string by processing escape sequences.
///
/// # Arguments
///
/// * `s` - The string content to unescape
///
/// # Returns
///
/// A new String with escape sequences processed
pub fn unescape_double_quoted(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('u') => {
                    let mut hex = String::new();
                    for _ in 0..4 {
                        if let Some(h) = chars.peek().copied() {
                            if h.is_ascii_hexdigit() {
                                hex.push(h);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    if hex.len() < 4 {
                        // Incomplete unicode escape: emit debug trace and return error marker
                        eprintln!("DEBUG: Incomplete unicode escape: \\u{}", hex);
                        result.push_str("[ERROR:INCOMPLETE_UNICODE_ESCAPE]");
                        break;
                    }
                    if let Ok(code) = u16::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code as u32) {
                            result.push(ch);
                            continue;
                        }
                    }
                    result.push('\\');
                    result.push('u');
                    result.push_str(&hex);
                }
                Some('U') => {
                    let mut hex = String::new();
                    for _ in 0..8 {
                        if let Some(h) = chars.peek().copied() {
                            if h.is_ascii_hexdigit() {
                                hex.push(h);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    if hex.len() < 8 {
                        eprintln!("DEBUG: Incomplete unicode escape: \\U{}", hex);
                        result.push_str("[ERROR:INCOMPLETE_UNICODE_ESCAPE]");
                        break;
                    }
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                            continue;
                        }
                    }
                    result.push('\\');
                    result.push('U');
                    result.push_str(&hex);
                }
                Some('x') => {
                    result.push('\\');
                    result.push('x');
                    for _ in 0..2 {
                        if let Some(h) = chars.peek().copied() {
                            if h.is_ascii_hexdigit() {
                                result.push(h);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                }
                Some('n') => {
                    result.push('\n');
                }
                Some('r') => {
                    result.push('\r');
                }
                Some('t') => {
                    result.push('\t');
                }
                Some('b') => {
                    result.push('\x08'); // backspace
                }
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('/') => result.push('/'),
                Some(' ') => result.push(' '),
                Some('0') => result.push('\0'),
                Some('a') => result.push('\x07'),
                Some('v') => result.push('\x0B'),
                Some('f') => result.push('\x0C'),
                Some('e') => result.push('\x1B'),
                Some('N') => result.push('\u{0085}'),
                Some('_') => result.push('\u{00A0}'),
                Some('L') => result.push('\u{2028}'),
                Some('P') => result.push('\u{2029}'),
                Some('\n') => {
                    while let Some(&c) = chars.peek() {
                        if c == crate::constants::CHAR_SPACE || c == crate::constants::CHAR_TAB {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                Some('\r') => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    while let Some(&c) = chars.peek() {
                        if c == crate::constants::CHAR_SPACE || c == crate::constants::CHAR_TAB {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                Some('\t') => {
                    while let Some(&c) = chars.peek() {
                        if c == crate::constants::CHAR_SPACE || c == crate::constants::CHAR_TAB {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                Some(other) => {
                    eprintln!("DEBUG: Invalid escape sequence: \\{}", other);
                    result.push('\\');
                    result.push(other);
                }
                None => {
                    eprintln!("DEBUG: Trailing backslash in string");
                    result.push('\\');
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_collect_until_and_read_line() {
        let mut buf = Buffer::new(b"hello, world\nnext");
        let s = collect_until(&mut buf, |c| c == ',');
        assert_eq!(s, "hello");

        buf.next();
        let line = read_line_trimmed_into_string(&mut buf);

        assert_eq!(line, "world");
    }
    #[test]
    fn test_skip_whitespace_and_comments() {
        let mut buf = Buffer::new(b"hello, world\n# comment\nnext");
        skip_whitespace_and_comments(&mut buf);
        assert_eq!(buf.current(), Some('h'));
    }
}
