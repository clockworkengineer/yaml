//! Module: stringify/json.rs

use crate::error::YamlError;
use crate::io::traits::IDestination;
use crate::nodes::node::*;
use crate::stringify::format::node_to_key_like_string;
use crate::stringify::serializer::{walk_node, FormatWriter, StreamResult};
use crate::utils::escape::escape_for_json;

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
    escape_for_json(s)
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

struct JsonWriter<'a> {
    dest: &'a mut dyn IDestination,
}

impl<'a> JsonWriter<'a> {
    fn new(dest: &'a mut dyn IDestination) -> Self {
        Self { dest }
    }
}

impl<'a> FormatWriter for JsonWriter<'a> {
    fn dest(&mut self) -> &mut dyn IDestination {
        self.dest
    }

    fn write_null(&mut self) -> StreamResult {
        self.dest.add_bytes("null");
        Ok(())
    }

    fn write_bool(&mut self, value: bool) -> StreamResult {
        if value {
            self.dest.add_bytes("true");
        } else {
            self.dest.add_bytes("false");
        }
        Ok(())
    }

    fn write_number(&mut self, num: &Numeric) -> StreamResult {
        match num {
            Numeric::Integer(i) => self.dest.add_bytes(&i.to_string()),
            Numeric::Float(f) => self.dest.add_bytes(&f.to_string()),
            Numeric::UInteger(u) => self.dest.add_bytes(&u.to_string()),
            Numeric::Byte(b) => self.dest.add_bytes(&b.to_string()),
            Numeric::Int32(i) => self.dest.add_bytes(&i.to_string()),
            Numeric::UInt32(u) => self.dest.add_bytes(&u.to_string()),
            Numeric::Int16(i) => self.dest.add_bytes(&i.to_string()),
            Numeric::UInt16(u) => self.dest.add_bytes(&u.to_string()),
            Numeric::Int8(i) => self.dest.add_bytes(&i.to_string()),
            Numeric::UInt8(u) => self.dest.add_bytes(&u.to_string()),
        }
        Ok(())
    }

    fn write_string(&mut self, value: &str) -> StreamResult {
        write_json_string(value, self.dest);
        Ok(())
    }

    fn start_array(&mut self, _len: usize) -> StreamResult {
        self.dest.add_byte(b'[');
        Ok(())
    }

    fn array_value_separator(&mut self, _index: usize) -> StreamResult {
        self.dest.add_byte(b',');
        Ok(())
    }

    fn end_array(&mut self) -> StreamResult {
        self.dest.add_byte(b']');
        Ok(())
    }

    fn start_set(&mut self, _len: usize) -> StreamResult {
        // sets are represented as arrays in JSON
        self.dest.add_byte(b'[');
        Ok(())
    }

    fn set_value_separator(&mut self, _index: usize) -> StreamResult {
        self.dest.add_byte(b',');
        Ok(())
    }

    fn end_set(&mut self) -> StreamResult {
        self.dest.add_byte(b']');
        Ok(())
    }

    fn start_mapping(&mut self, _len: usize) -> StreamResult {
        self.dest.add_byte(b'{');
        Ok(())
    }

    fn write_mapping_key(&mut self, key: &Node) -> StreamResult {
        let key_str = node_to_key_like_string(key);
        write_json_string(&key_str, self.dest);
        Ok(())
    }

    fn mapping_key_value_separator(&mut self) -> StreamResult {
        self.dest.add_byte(b':');
        Ok(())
    }

    fn mapping_entry_separator(&mut self, _index: usize) -> StreamResult {
        self.dest.add_byte(b',');
        Ok(())
    }

    fn end_mapping(&mut self) -> StreamResult {
        self.dest.add_byte(b'}');
        Ok(())
    }

    fn write_comment(&mut self, _comment: &str) -> StreamResult {
        // comments are represented as nulls in legacy JSON backend
        self.dest.add_bytes("null");
        Ok(())
    }

    fn start_document(&mut self, index: usize, total: usize) -> StreamResult {
        if total <= 1 {
            return Ok(());
        }
        if index == 0 {
            self.dest.add_byte(b'[');
        }
        if index > 0 {
            self.dest.add_byte(b',');
        }
        Ok(())
    }

    fn end_document(&mut self, index: usize, total: usize) -> StreamResult {
        if total > 1 && index + 1 == total {
            self.dest.add_byte(b']');
        }
        Ok(())
    }
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
pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), YamlError> {
    let mut writer = JsonWriter::new(destination);
    walk_node(&mut writer, node).map_err(Into::into)
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
) -> Result<(), YamlError> {
    fn helper(
        node: &Node,
        destination: &mut dyn IDestination,
        spaces: usize,
        level: usize,
    ) -> Result<(), YamlError> {
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
                Numeric::UInt8(u) => destination.add_bytes(&u.to_string()),
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
                    let key_str = node_to_key_like_string(k);
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
