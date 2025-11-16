//! Module: utils/mod.rs

/// Performance measurement and optimization utilities
pub mod performance;

/// String interning for memory optimization
#[cfg(feature = "alloc")]
pub mod string_interner;

/// Performance optimization utilities (lazy evaluation, capacity hints, zero-copy)
#[cfg(feature = "alloc")]
pub mod optimization;

use crate::constants::{CHAR_HASH, CHAR_NEWLINE};
use crate::io::traits::ISource;
use crate::{Node, Numeric};

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
pub fn collect_until<F>(source: &mut dyn ISource, mut stop_pred: F) -> String
where
    F: FnMut(char) -> bool,
{
    let mut out = String::new();
    while let Some(c) = source.current() {
        if stop_pred(c) {
            break;
        }
        out.push(c);
        source.next();
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
pub fn skip_whitespace_and_comments(source: &mut dyn ISource) {
    loop {
        while let Some(c) = source.current() {
            if source.is_whitespace(c) {
                source.next();
            } else {
                break;
            }
        }
        if source.current() == Some(CHAR_HASH) {
            while let Some(c) = source.current() {
                if c == CHAR_NEWLINE {
                    break;
                }
                source.next();
            }

            continue;
        }
        break;
    }
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

/// Consumes an inline comment and any following newline and whitespace.
///
/// If the current character is a hash (#), consumes all characters until a newline,
/// then consumes the newline and any whitespace that follows. If the current character
/// is not a hash, this function returns immediately without consuming anything.
///
/// # Arguments
///
/// * `source` - A mutable reference to a source implementing ISource trait
pub fn consume_inline_comment_and_newline(source: &mut dyn ISource) {
    if source.current() != Some(CHAR_HASH) {
        return;
    }

    while let Some(c) = source.current() {
        if c == CHAR_NEWLINE {
            break;
        }
        source.next();
    }

    if source.current() == Some(CHAR_NEWLINE) {
        source.next();
    }

    while let Some(c) = source.current() {
        if source.is_whitespace(c) {
            source.next();
        } else {
            break;
        }
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

/// Processes escape sequences in a double-quoted string.
///
/// Handles various escape sequences including Unicode escapes (\u), hex escapes (\x),
/// and standard escape characters (\n, \r, \t, \b, \", \\). Unicode escapes are
/// processed but not fully decoded (preserves as-is for most cases).
///
/// # Arguments
///
/// * `s` - A string slice containing the potentially escaped string
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
                    if hex.len() == 4 {
                        if let Ok(code) = u16::from_str_radix(&hex, 16) {
                            if let Some(ch) = char::from_u32(code as u32) {
                                result.push(ch);
                                continue;
                            }
                        }
                    }

                    result.push('\\');
                    result.push('u');
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
                    result.push('\\');
                    result.push('n');
                }
                Some('r') => {
                    result.push('\\');
                    result.push('r');
                }
                Some('t') => {
                    result.push('\\');
                    result.push('t');
                }
                Some('b') => {
                    result.push('\\');
                    result.push('b');
                }

                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),

                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => {
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
