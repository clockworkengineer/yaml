use crate::io::destinations::buffer::Buffer as BufferDestination;
use crate::io::traits::IDestination;
use crate::nodes::node::*;
use crate::stringify::default::stringify as yaml_stringify;

// Basic JSON string escaper. Not fully comprehensive for all Unicode
// escaping, but sufficient for typical ASCII/control character escaping
// used in our tests (quotes, backslashes, control chars).
fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            other => out.push(other),
        }
    }
    out
}

fn write_json_string(s: &str, destination: &mut dyn IDestination) {
    destination.add_byte(b'"');
    destination.add_bytes(&escape_json_string(s));
    destination.add_byte(b'"');
}

fn node_to_key_string(key: &Node) -> Result<String, String> {
    match key {
        Node::Str(s, _, _) => Ok(s.clone()),
        Node::Number(num) => match num {
            Numeric::Integer(i) => Ok(i.to_string()),
            Numeric::Float(f) => Ok(f.to_string()),
            Numeric::UInteger(u) => Ok(u.to_string()),
            Numeric::Byte(b) => Ok(b.to_string()),
            Numeric::Int32(i) => Ok(i.to_string()),
            Numeric::UInt32(u) => Ok(u.to_string()),
            Numeric::Int16(i) => Ok(i.to_string()),
            Numeric::UInt16(u) => Ok(u.to_string()),
            Numeric::Int8(i) => Ok(i.to_string()),
        },
        Node::Boolean(b) => Ok((if *b { "true" } else { "false" }).to_string()),
        Node::None => Ok("".to_string()),
        _ => {
            // Fall back to YAML stringify for complex keys, then use that
            // textual representation as the JSON key string.
            let mut buf = BufferDestination::new();
            yaml_stringify(key, &mut buf).map_err(|e| e)?;
            Ok(buf.to_string())
        }
    }
}

fn stringify_node(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::None => destination.add_bytes("null"),
        Node::Boolean(b) => destination.add_bytes(if *b { "true" } else { "false" }),
        Node::Str(s, _, _) => write_json_string(s, destination),
        Node::Number(num) => match num {
            Numeric::Integer(i) => destination.add_bytes(&i.to_string()),
            Numeric::Float(f) => destination.add_bytes(&f.to_string()),
            Numeric::UInteger(u) => destination.add_bytes(&u.to_string()),
            Numeric::Byte(b) => destination.add_bytes(&b.to_string()),
            Numeric::Int32(i) => destination.add_bytes(&i.to_string()),
            Numeric::UInt32(u) => destination.add_bytes(&u.to_string()),
            Numeric::Int16(i) => destination.add_bytes(&i.to_string()),
            Numeric::UInt16(u) => destination.add_bytes(&u.to_string()),
            Numeric::Int8(i) => destination.add_bytes(&i.to_string()),
        },
        Node::Array(items) => {
            destination.add_byte(b'[');
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    destination.add_byte(b',');
                }
                stringify_node(it, destination)?;
            }
            destination.add_byte(b']');
        }
        Node::Mapping(pairs) => {
            destination.add_byte(b'{');
            let mut first = true;
            for (k, v) in pairs.iter() {
                // Skip comment keys if any — but ensure iteration stays deterministic
                let key_str = node_to_key_string(k)?;
                if !first {
                    destination.add_byte(b',');
                }
                first = false;
                write_json_string(&key_str, destination);
                destination.add_byte(b':');
                stringify_node(v, destination)?;
            }
            destination.add_byte(b'}');
        }
        Node::Document(nodes) => {
            if nodes.len() == 1 {
                stringify_node(&nodes[0], destination)?;
            } else {
                destination.add_byte(b'[');
                for (i, n) in nodes.iter().enumerate() {
                    if i > 0 {
                        destination.add_byte(b',');
                    }
                    stringify_node(n, destination)?;
                }
                destination.add_byte(b']');
            }
        }
        Node::Tagged(inner, _tag) => stringify_node(inner, destination)?,
        Node::Anchored(inner, _name) => stringify_node(inner, destination)?,
        Node::Alias(_name) => destination.add_bytes("null"),
        Node::Documents(docs) => {
            destination.add_byte(b'[');
            for (i, d) in docs.iter().enumerate() {
                if i > 0 {
                    destination.add_byte(b',');
                }
                stringify_node(d, destination)?;
            }
            destination.add_byte(b']');
        }
        Node::Comment(_) => {
            // JSON has no comments; skip comments by emitting null for standalone comment nodes
            destination.add_bytes("null");
        }
    }
    Ok(())
}

pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    stringify_node(node, destination)
}

/// Pretty-print JSON with the given number of spaces per indent level.
pub fn stringify_pretty(
    node: &Node,
    destination: &mut dyn IDestination,
    spaces_per_indent: usize,
) -> Result<(), String> {
    fn helper(
        node: &Node,
        destination: &mut dyn IDestination,
        spaces: usize,
        level: usize,
    ) -> Result<(), String> {
        match node {
            Node::None => destination.add_bytes("null"),
            Node::Boolean(b) => destination.add_bytes(if *b { "true" } else { "false" }),
            Node::Str(s, _, _) => write_json_string(s, destination),
            Node::Number(num) => match num {
                Numeric::Integer(i) => destination.add_bytes(&i.to_string()),
                Numeric::Float(f) => destination.add_bytes(&f.to_string()),
                Numeric::UInteger(u) => destination.add_bytes(&u.to_string()),
                Numeric::Byte(b) => destination.add_bytes(&b.to_string()),
                Numeric::Int32(i) => destination.add_bytes(&i.to_string()),
                Numeric::UInt32(u) => destination.add_bytes(&u.to_string()),
                Numeric::Int16(i) => destination.add_bytes(&i.to_string()),
                Numeric::UInt16(u) => destination.add_bytes(&u.to_string()),
                Numeric::Int8(i) => destination.add_bytes(&i.to_string()),
            },
            Node::Array(items) => {
                if items.is_empty() {
                    destination.add_bytes("[]");
                    return Ok(());
                }
                destination.add_byte(b'[');
                destination.add_byte(b'\n');
                for (i, it) in items.iter().enumerate() {
                    let indent = " ".repeat(spaces * (level + 1));
                    destination.add_bytes(&indent);
                    helper(it, destination, spaces, level + 1)?;
                    if i + 1 < items.len() {
                        destination.add_byte(b',');
                    }
                    destination.add_byte(b'\n');
                }
                let indent = " ".repeat(spaces * level);
                destination.add_bytes(&indent);
                destination.add_byte(b']');
            }
            Node::Mapping(pairs) => {
                if pairs.is_empty() {
                    destination.add_bytes("{}");
                    return Ok(());
                }
                destination.add_byte(b'{');
                destination.add_byte(b'\n');
                for (i, (k, v)) in pairs.iter().enumerate() {
                    let indent = " ".repeat(spaces * (level + 1));
                    destination.add_bytes(&indent);
                    let key_str = node_to_key_string(k)?;
                    write_json_string(&key_str, destination);
                    destination.add_bytes(": ");
                    helper(v, destination, spaces, level + 1)?;
                    if i + 1 < pairs.len() {
                        destination.add_byte(b',');
                    }
                    destination.add_byte(b'\n');
                }
                let indent = " ".repeat(spaces * level);
                destination.add_bytes(&indent);
                destination.add_byte(b'}');
            }
            Node::Document(nodes) => {
                if nodes.len() == 1 {
                    helper(&nodes[0], destination, spaces, level)?;
                } else {
                    destination.add_byte(b'[');
                    destination.add_byte(b'\n');
                    for (i, n) in nodes.iter().enumerate() {
                        let indent = " ".repeat(spaces * (level + 1));
                        destination.add_bytes(&indent);
                        helper(n, destination, spaces, level + 1)?;
                        if i + 1 < nodes.len() {
                            destination.add_byte(b',');
                        }
                        destination.add_byte(b'\n');
                    }
                    let indent = " ".repeat(spaces * level);
                    destination.add_bytes(&indent);
                    destination.add_byte(b']');
                }
            }
            Node::Tagged(inner, _tag) => helper(inner, destination, spaces, level)?,
            Node::Anchored(inner, _name) => helper(inner, destination, spaces, level)?,
            Node::Alias(_name) => destination.add_bytes("null"),
            Node::Documents(docs) => {
                destination.add_byte(b'[');
                destination.add_byte(b'\n');
                for (i, d) in docs.iter().enumerate() {
                    let indent = " ".repeat(spaces * (level + 1));
                    destination.add_bytes(&indent);
                    helper(d, destination, spaces, level + 1)?;
                    if i + 1 < docs.len() {
                        destination.add_byte(b',');
                    }
                    destination.add_byte(b'\n');
                }
                let indent = " ".repeat(spaces * level);
                destination.add_bytes(&indent);
                destination.add_byte(b']');
            }
            Node::Comment(_) => destination.add_bytes("null"),
        }
        Ok(())
    }

    helper(node, destination, spaces_per_indent, 0)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::buffer::Buffer as BufferDestination;
    use crate::nodes::node::{BlockStyle, Node, Numeric, QuoteType};

    #[test]
    fn test_json_basic() {
        let mut buf = BufferDestination::new();
        let n = Node::Str(
            "hello\nworld".to_string(),
            QuoteType::Unquoted,
            BlockStyle::None,
        );
        stringify(&n, &mut buf).unwrap();
        assert_eq!(buf.to_string(), "\"hello\\nworld\"");

        buf.clear();
        let ni = Node::Number(Numeric::Integer(10));
        stringify(&ni, &mut buf).unwrap();
        assert_eq!(buf.to_string(), "10");
    }
}
