use crate::io::traits::ISource;
use crate::parser::constants::{CHAR_HASH, CHAR_NEWLINE};

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

// Read characters until newline (or end) and return trimmed string; leaves cursor at the newline (not consumed)
pub fn read_line_trimmed_into_string(source: &mut dyn ISource) -> String {
    let s = collect_until(source, |c| c == CHAR_NEWLINE);
    s.trim().to_string()
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
        // consume comma
        buf.next();
        let line = read_line_trimmed_into_string(&mut buf);
        // read_line_trimmed_into_string trims whitespace
        assert_eq!(line, "world");
    }
    
}
