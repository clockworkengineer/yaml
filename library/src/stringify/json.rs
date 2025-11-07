//! Module: stringify/json.rs

use crate::io::destinations::buffer::Buffer as BufferDestination;
use crate::io::traits::IDestination;
use crate::nodes::node::*;
use crate::stringify::default::stringify as yaml_stringify;

/// Escapes special characters in a string for JSON representation.
///
/// Handles JSON-specific escape sequences including quotes, backslashes,
/// newlines, and control characters. Converts control characters to
/// Unicode escape sequences.
///
/// # Arguments
///
/// * `s` - The string to escape
///
/// # Returns
///
/// A new String with proper JSON escape sequences
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

/// Writes a JSON-escaped string to the destination with surrounding quotes.
///
/// Combines escaping and quoting into a single operation for efficiency.
///
/// # Arguments
///
/// * `s` - The string to escape and write
/// * `destination` - The output destination
fn write_json_string(s: &str, destination: &mut dyn IDestination) {
    destination.add_byte(b'"');
    destination.add_bytes(&escape_json_string(s));
    destination.add_byte(b'"');
}

/// Converts a YAML node to a string representation suitable for use as a JSON key.
///
/// JSON keys must be strings, so this function converts various node types
/// to appropriate string representations for use as object keys.
///
/// # Arguments
///
/// * `key` - The Node to convert to a key string
///
/// # Returns
///
/// Result containing the key string or an error for invalid key types
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
            let mut buf = BufferDestination::new();
            yaml_stringify(key, &mut buf).map_err(|e| e)?;
            Ok(buf.to_string())
        }
    }
}

/// Recursively stringifies a YAML node to JSON format.
///
/// Converts YAML nodes to their JSON equivalents, handling type mappings
/// and structural differences between YAML and JSON formats.
///
/// # Arguments
///
/// * `node` - The Node to convert to JSON
/// * `destination` - The output destination for the JSON content
///
/// # Returns
///
/// Result indicating success or an error string
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
        Node::Set(items) => {
            // Represent sets as JSON arrays in JSON output
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
            destination.add_bytes("null");
        }
    }
    Ok(())
}

/// stringify

/// Converts YAML nodes to compact JSON format.
///
/// Main entry point for JSON stringification. Converts YAML document
/// structures to valid JSON, handling multi-document streams appropriately.
///
/// # Arguments
///
/// * `node` - The root Node to convert to JSON
/// * `destination` - The output destination for the JSON content
///
/// # Returns
///
/// Result indicating success or an error string
pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    stringify_node(node, destination)
}

/// Converts YAML nodes to pretty-printed JSON format with indentation.
///
/// Similar to stringify() but adds proper indentation and newlines
/// for human-readable JSON output.
///
/// # Arguments
///
/// * `node` - The root Node to convert to JSON
/// * `destination` - The output destination for the JSON content
/// * `spaces_per_indent` - The number of spaces per indentation level
///
/// # Returns
///
/// Result indicating success or an error string
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
            Node::Set(items) => {
                // Represent sets as JSON arrays in JSON output
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
