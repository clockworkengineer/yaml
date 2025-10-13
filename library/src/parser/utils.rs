use crate::io::traits::ISource;
use crate::nodes::node::Node;
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

// Convert a small Node into a string suitable for a map key
pub fn node_to_map_key(node: &Node) -> String {
    match node {
        Node::Array(items) => {
            let parts: Vec<String> = items
                .iter()
                .map(|n| match n {
                    Node::Str(s, _qt) => s.clone(),
                    Node::Number(num) => format!("{:?}", num),
                    Node::Boolean(b) => b.to_string(),
                    _ => format!("{:?}", n),
                })
                .collect();
            format!("[{}]", parts.join(", "))
        }
        Node::Str(s, _qt) => s.clone(),
        _ => format!("{:?}", node),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;
    use crate::nodes::node::{Node, Numeric, QuoteType};

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
    fn test_node_to_map_key() {
        let items = vec![
            Node::Str("a".to_string(), QuoteType::Unquoted),
            Node::Number(Numeric::Integer(1)),
            Node::Boolean(true),
        ];
        let key = node_to_map_key(&Node::Array(items));
        // node_to_map_key uses Debug formatting for Numeric variants
        assert_eq!(key, "[a, Integer(1), true]");
    }
}
