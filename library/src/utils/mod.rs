use crate::constants::{CHAR_HASH, CHAR_NEWLINE};
use crate::io::traits::ISource;
use crate::{Node, Numeric};

// Collect characters until a stop predicate triggers; does not consume the stop char
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

// Skip whitespace and optional single-line comments starting with '#'
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
            // skip until newline and leave cursor at newline
            while let Some(c) = source.current() {
                if c == CHAR_NEWLINE {
                    break;
                }
                source.next();
            }
            // continue loop to skip whitespace after comment
            continue;
        }
        break;
    }
}

// Skip characters until a newline is encountered; consumes the newline if present
pub fn skip_until_newline(source: &mut dyn ISource) {
    while let Some(c) = source.current() {
        if c == CHAR_NEWLINE {
            source.next();
            break;
        }
        source.next();
    }
}

// Consume an inline '#' comment to end-of-line, then consume a single newline if present,
// and finally skip any following whitespace. Leaves the cursor positioned at the first
// non-whitespace character after the newline, or at EOF. If the current character is
// not '#', this function is a no-op.
pub fn consume_inline_comment_and_newline(source: &mut dyn ISource) {
    if source.current() != Some(CHAR_HASH) {
        return;
    }
    // Consume the comment at the end of the line
    while let Some(c) = source.current() {
        if c == CHAR_NEWLINE {
            break;
        }
        source.next();
    }
    // Consume a single newline if present
    if source.current() == Some(CHAR_NEWLINE) {
        source.next();
    }
    // Skip trailing whitespace
    while let Some(c) = source.current() {
        if source.is_whitespace(c) {
            source.next();
        } else {
            break;
        }
    }
}

// Read characters until the newline (or end) and return trimmed string; leaves cursor at the newline (not consumed)
pub fn read_line_trimmed_into_string(source: &mut dyn ISource) -> String {
    let s = collect_until(source, |c| c == CHAR_NEWLINE);
    // If there's an inline comment starting with '#', strip it and return the
    // part before it. This ensures callers that want the line content without
    // trailing comments get a clean string.
    if let Some(pos) = s.find(CHAR_HASH) {
        return s[..pos].trim().to_string();
    }
    s.trim().to_string()
}

// Helper: produce a compact inline representation of a Node suitable for
// turning into a string key. Handles sequences and mappings recursively.
pub fn node_to_inline_string(node: &Node) -> String {
    match node {
        Node::Str(s, _, _) => s.clone(),
        Node::Number(Numeric::Integer(i)) => i.to_string(),
        Node::Number(Numeric::Float(f)) => f.to_string(),
        Node::Boolean(b) => b.to_string(),
        Node::Array(items) => {
            let parts: Vec<String> = items.iter().map(|it| node_to_inline_string(it)).collect();
            format!("[{}]", parts.join(", "))
        }
        Node::Mapping(pairs) => {
            let parts: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("{}: {}", node_to_inline_string(k), node_to_inline_string(v)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        _ => format!("{:?}", node),
    }
}

pub fn unescape_double_quoted(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                // Decode Unicode escapes \uXXXX into actual characters
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
                    // Fallback: preserve literally if malformed
                    result.push('\\');
                    result.push('u');
                    result.push_str(&hex);
                }
                // Preserve hex escapes literally (e.g., \x13)
                Some('x') => {
                    result.push('\\');
                    result.push('x');
                    // copy up to two hex digits literally
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
                // Keep standard escapes as literal backslash+letter so they survive stringify
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
                // Unescape quote and backslash to their literal characters
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                // Any other escape: keep it literally
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => {
                    // Trailing backslash, keep it
                    result.push('\\');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}
// Character constants imported from `crate::parser::constants`

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_collect_until_and_read_line() {
        let mut buf = Buffer::new(b"hello, world\nnext");
        let s = collect_until(&mut buf, |c| c == ',');
        assert_eq!(s, "hello");
        // consume comma
        buf.next();
        let line = read_line_trimmed_into_string(&mut buf);
        // read_line_trimmed_into_string trims whitespace
        assert_eq!(line, "world");
    }
    #[test]
    fn test_skip_whitespace_and_comments() {
        let mut buf = Buffer::new(b"hello, world\n# comment\nnext");
        skip_whitespace_and_comments(&mut buf);
        assert_eq!(buf.current(), Some('h'));
    }
}
